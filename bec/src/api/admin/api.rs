//! Админ-API: управление каталогом автомобилей.
//!
//! Все маршруты защищены токеном (`X-Admin-Token`, см. `api::router::require_admin`)
//! и меняют данные — добавляют, удаляют авто, переводят в «продано» и загружают
//! фотографии. Публичный API (`/api/*`) остаётся только для чтения.

use axum::extract::{Multipart, Path as UrlPath, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::api::admin::media::{
    read_images, remove_empty_dir, remove_image_file, save_image, IMAGES_PREFIX,
};
use crate::api::models::{
    bad_request, internal_error, not_found, ok, ApiResult, Car, NewCar, StatusUpdate, VisitStats,
};
use crate::api::AppState;
use crate::core::models::{AdminCarFilter, CarUpdate, CAR_STATUSES};

/// Параметры списка админ-API: статус, поиск, сортировка, пагинация.
#[derive(Debug, Deserialize)]
pub struct AdminCarsQuery {
    pub status: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Параметры удаления фото: путь к файлу, который раздаёт сервер.
#[derive(Debug, Deserialize)]
pub struct ImageQuery {
    pub url: String,
}

/// GET /api/admin/cars — все автомобили, включая проданные.
///
/// Отличие от публичного `/api/cars`: без фильтра отдаются и проданные —
/// админке нужен полный список.
pub async fn list_handler(
    State(state): State<AppState>,
    Query(query): Query<AdminCarsQuery>,
) -> Result<Json<Vec<Car>>, ApiResult> {
    let status = match query.status.as_deref().filter(|value| !value.is_empty()) {
        None => None,
        Some(status) if CAR_STATUSES.contains(&status) => Some(status.to_string()),
        Some(other) => {
            tracing::warn!(status = %other, "Недопустимый фильтр status в админ-API");
            return Err(bad_request(
                "Недопустимый статус. Разрешены: stock, transit, order, sold",
            ));
        }
    };

    let filter = AdminCarFilter {
        status,
        query: query.q,
        sort: query.sort,
        limit: query.limit.unwrap_or(50).clamp(1, 100),
        offset: query.offset.unwrap_or(0).max(0),
    };

    match state.db.admin_filter_cars(&filter).await {
        Ok(cars) => Ok(Json(cars)),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить список авто (админ-API)");
            Err(internal_error())
        }
    }
}

/// GET /api/admin/stats/visits — счётчик посещений сайта.
///
/// Числа доступны только с админ-токеном: на публичной странице и в публичном
/// `/api/*` их нет — счётчик не показывается посетителям.
pub async fn visits_handler(State(state): State<AppState>) -> Result<Json<VisitStats>, ApiResult> {
    match state.db.site_visits().await {
        Ok(stats) => Ok(Json(stats)),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить счётчик посещений");
            Err(internal_error())
        }
    }
}

/// POST /api/admin/cars — добавить автомобиль.
pub async fn create_handler(
    State(state): State<AppState>,
    Json(new): Json<NewCar>,
) -> Result<(StatusCode, Json<Car>), ApiResult> {
    validate_new_car(&new)?;

    match state.db.create_car(&new).await {
        Ok(car) => {
            tracing::info!(id = car.id, lot = %car.lot_number, "Автомобиль добавлен");
            Ok((StatusCode::CREATED, Json(car)))
        }
        Err(error) => {
            // Частая причина — дубликат lot_number (колонка UNIQUE). Клиенту
            // детали не раскрываем, но в логе они есть.
            tracing::error!(error = %error, "Не удалось создать автомобиль");
            Err(internal_error())
        }
    }
}

/// PATCH /api/admin/cars/{id} — изменить характеристики автомобиля.
pub async fn update_handler(
    State(state): State<AppState>,
    UrlPath(id): UrlPath<i64>,
    Json(update): Json<CarUpdate>,
) -> Result<Json<Car>, ApiResult> {
    validate_car_update(&update)?;

    match state.db.update_car(id, &update).await {
        Ok(Some(car)) => {
            tracing::info!(id, lot = %car.lot_number, "Автомобиль изменён");
            Ok(Json(car))
        }
        Ok(None) => Err(not_found("Автомобиль не найден")),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось изменить автомобиль");
            Err(internal_error())
        }
    }
}

/// PATCH /api/admin/cars/{id}/status — сменить статус (в т.ч. «продано»).
pub async fn set_status_handler(
    State(state): State<AppState>,
    UrlPath(id): UrlPath<i64>,
    Json(update): Json<StatusUpdate>,
) -> Result<Json<Car>, ApiResult> {
    if !CAR_STATUSES.contains(&update.status.as_str()) {
        return Err(bad_request(
            "Недопустимый статус. Разрешены: stock, transit, order, sold",
        ));
    }

    match state.db.set_car_status(id, &update).await {
        Ok(Some(car)) => {
            tracing::info!(id, status = %update.status, "Статус автомобиля изменён");
            Ok(Json(car))
        }
        Ok(None) => Err(not_found("Автомобиль не найден")),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось изменить статус автомобиля");
            Err(internal_error())
        }
    }
}

/// DELETE /api/admin/cars/{id} — удалить автомобиль вместе с фотографиями.
pub async fn delete_handler(State(state): State<AppState>, UrlPath(id): UrlPath<i64>) -> ApiResult {
    match state.db.delete_car(id).await {
        Ok(Some(urls)) => {
            // Строки в БД уже удалены — убираем файлы, чтобы не копить мусор.
            for url in &urls {
                remove_image_file(&state, url).await;
            }
            remove_empty_dir(&state, &urls).await;

            tracing::info!(id, photos = urls.len(), "Автомобиль удалён");
            ok(format!(
                "Автомобиль удалён, снято фотографий: {}",
                urls.len()
            ))
        }
        Ok(None) => not_found("Автомобиль не найден"),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось удалить автомобиль");
            internal_error()
        }
    }
}

/// POST /api/admin/cars/{id}/images — загрузить фотографию (multipart, поле-файл).
pub async fn upload_image_handler(
    State(state): State<AppState>,
    UrlPath(id): UrlPath<i64>,
    multipart: Multipart,
) -> Result<Json<Car>, ApiResult> {
    let Some(car) = fetch_car(&state, id).await? else {
        return Err(not_found("Автомобиль не найден"));
    };

    let images = read_images(multipart).await.map_err(bad_request)?;

    // JSON-эндпоинт принимает одно фото (как и раньше), HTML-админка — несколько.
    let (extension, bytes) = &images[0];
    let saved = save_image(&state, &car, extension, bytes)
        .await
        .map_err(bad_request)?;

    if let Err(error) = state.db.add_car_image(id, &saved.url).await {
        tracing::error!(error = %error, "Не удалось сохранить фото в БД");
        // Файл уже на диске, но без записи в БД он «потерян» — убираем за собой.
        crate::api::admin::media::remove_saved(&saved).await;
        return Err(internal_error());
    }

    tracing::info!(id, url = %saved.url, "Фото добавлено");
    fetch_car_json(&state, id).await
}

/// DELETE /api/admin/cars/{id}/images?url=/images/... — удалить одно фото.
pub async fn delete_image_handler(
    State(state): State<AppState>,
    UrlPath(id): UrlPath<i64>,
    Query(query): Query<ImageQuery>,
) -> Result<Json<Car>, ApiResult> {
    let url = query.url.trim().to_string();

    // Путь приходит от клиента: разрешаем только наши фото внутри /images/.
    if !url.starts_with(IMAGES_PREFIX) || url.contains("..") {
        return Err(bad_request("Некорректный путь фото"));
    }

    match state.db.delete_car_image(id, &url).await {
        Ok(true) => {
            remove_image_file(&state, &url).await;
            tracing::info!(id, url = %url, "Фото удалено");
            fetch_car_json(&state, id).await
        }
        Ok(false) => Err(not_found("Фото не найдено")),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось удалить фото");
            Err(internal_error())
        }
    }
}

/// Проверяет обязательные поля и рамки значений перед вставкой в БД.
fn validate_new_car(new: &NewCar) -> Result<(), ApiResult> {
    if new.brand.trim().is_empty() {
        return Err(bad_request("Не указана марка"));
    }
    if new.model.trim().is_empty() {
        return Err(bad_request("Не указана модель"));
    }
    if !(1950..=2100).contains(&new.year) {
        return Err(bad_request("Некорректный год выпуска"));
    }
    if new.mileage < 0 {
        return Err(bad_request("Пробег не может быть отрицательным"));
    }
    if new.price_rub < 0 {
        return Err(bad_request("Цена не может быть отрицательной"));
    }
    if new.engine.trim().is_empty() {
        return Err(bad_request("Не указан двигатель"));
    }
    if new.transmission.trim().is_empty() {
        return Err(bad_request("Не указана коробка передач"));
    }
    if new.drive.trim().is_empty() {
        return Err(bad_request("Не указан привод"));
    }
    if let Some(status) = new.status.as_deref() {
        if !CAR_STATUSES.contains(&status) {
            return Err(bad_request(
                "Недопустимый статус. Разрешены: stock, transit, order, sold",
            ));
        }
    }
    Ok(())
}

/// Проверяет характеристики при редактировании.
fn validate_car_update(update: &CarUpdate) -> Result<(), ApiResult> {
    if update.lot_number.trim().is_empty() {
        return Err(bad_request("Не указан номер лота"));
    }
    if update.brand.trim().is_empty() {
        return Err(bad_request("Не указана марка"));
    }
    if update.model.trim().is_empty() {
        return Err(bad_request("Не указана модель"));
    }
    if !(1950..=2100).contains(&update.year) {
        return Err(bad_request("Некорректный год выпуска"));
    }
    if update.mileage < 0 {
        return Err(bad_request("Пробег не может быть отрицательным"));
    }
    if update.price_rub < 1 {
        return Err(bad_request("Цена должна быть положительной"));
    }
    if update.engine.trim().is_empty()
        || update.transmission.trim().is_empty()
        || update.drive.trim().is_empty()
    {
        return Err(bad_request("Заполните двигатель, коробку передач и привод"));
    }
    Ok(())
}

/// Читает авто или возвращает 404/500 — общий шаг для загрузки и удаления фото.
async fn fetch_car(state: &AppState, id: i64) -> Result<Option<Car>, ApiResult> {
    state.db.car_by_id(id).await.map_err(|error| {
        tracing::error!(error = %error, "Не удалось получить автомобиль");
        internal_error()
    })
}

/// Возвращает актуальный автомобиль после изменения (JSON или ошибку).
async fn fetch_car_json(state: &AppState, id: i64) -> Result<Json<Car>, ApiResult> {
    match fetch_car(state, id).await? {
        Some(car) => Ok(Json(car)),
        None => Err(not_found("Автомобиль не найден")),
    }
}
