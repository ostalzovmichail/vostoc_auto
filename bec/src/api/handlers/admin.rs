//! Админ-API: управление каталогом автомобилей.
//!
//! Все маршруты защищены токеном (`X-Admin-Token`, см. `api::router::require_admin`)
//! и меняют данные — добавляют, удаляют авто, переводят в «продано» и загружают
//! фотографии. Публичный API (`/api/*`) остаётся только для чтения.

use std::path::{Component, Path, PathBuf};

use axum::extract::{Multipart, Path as UrlPath, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::api::handlers::cars::{normalize_paging, CarsQuery};
use crate::api::models::{
    bad_request, internal_error, not_found, ok, ApiResult, Car, NewCar, StatusUpdate, VisitStats,
};
use crate::api::AppState;
use crate::core::models::CAR_STATUSES;

/// Максимальный размер загружаемого фото.
const MAX_IMAGE_BYTES: usize = 15 * 1024 * 1024;

/// Префикс, под которым раздаются фотографии (совпадает с `static_routes`).
const IMAGES_PREFIX: &str = "/images/";

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
    Query(query): Query<CarsQuery>,
) -> Result<Json<Vec<Car>>, ApiResult> {
    let status = match query.status.as_deref() {
        None => None,
        Some(status) if CAR_STATUSES.contains(&status) => Some(status),
        Some(other) => {
            tracing::warn!(status = %other, "Недопустимый фильтр status в админ-API");
            return Err(bad_request(
                "Недопустимый статус. Разрешены: stock, transit, order, sold",
            ));
        }
    };
    let (limit, offset) = normalize_paging(&query);

    match state.db.admin_list_cars(status, limit, offset).await {
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
    mut multipart: Multipart,
) -> Result<Json<Car>, ApiResult> {
    let Some(car) = fetch_car(&state, id).await? else {
        return Err(not_found("Автомобиль не найден"));
    };

    let mut uploaded: Option<(String, Vec<u8>)> = None;
    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(error) => {
                tracing::warn!(error = %error, "Некорректный multipart-запрос");
                return Err(bad_request("Некорректный multipart-запрос"));
            }
        };

        // Поле без имени файла — не файл (например, обычное текстовое): пропускаем.
        let Some(file_name) = field.file_name().map(str::to_string) else {
            continue;
        };
        let content_type = field.content_type().map(str::to_string);

        let Some(extension) = image_extension(content_type.as_deref(), &file_name) else {
            return Err(bad_request("Поддерживаются только JPEG, PNG и WebP"));
        };

        let bytes = match field.bytes().await {
            Ok(bytes) => bytes,
            Err(error) => {
                tracing::warn!(error = %error, "Не удалось прочитать загружаемый файл");
                return Err(bad_request("Не удалось прочитать файл"));
            }
        };

        if bytes.is_empty() {
            return Err(bad_request("Пустой файл"));
        }
        if bytes.len() > MAX_IMAGE_BYTES {
            return Err(bad_request("Файл больше 15 МБ"));
        }

        uploaded = Some((extension, bytes.to_vec()));
        break;
    }

    let Some((extension, bytes)) = uploaded else {
        return Err(bad_request("В запросе нет файла"));
    };

    // Каталог группируем по марке/модели/лоту — файлы одного авто лежат рядом.
    // slugify оставляет только ASCII-буквы, цифры и дефисы, поэтому выход за
    // пределы каталога фото исключён ещё на этапе имени.
    let slug = slugify(&format!("{}-{}-{}", car.brand, car.model, car.lot_number));
    let directory = Path::new(&state.config.images_dir).join(&slug);
    if let Err(error) = tokio::fs::create_dir_all(&directory).await {
        tracing::error!(error = %error, "Не удалось создать каталог для фото");
        return Err(internal_error());
    }

    let file_name = format!("{slug}-{}.{extension}", image_stamp());
    let path = directory.join(&file_name);
    if let Err(error) = tokio::fs::write(&path, &bytes).await {
        tracing::error!(error = %error, path = %path.display(), "Не удалось записать фото");
        return Err(internal_error());
    }

    let url = format!("/images/{slug}/{file_name}");
    if let Err(error) = state.db.add_car_image(id, &url).await {
        tracing::error!(error = %error, "Не удалось сохранить фото в БД");
        // Файл уже на диске, но без записи в БД он «потерян» — убираем за собой.
        let _ = tokio::fs::remove_file(&path).await;
        return Err(internal_error());
    }

    tracing::info!(id, url = %url, "Фото добавлено");
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

/// Определяет расширение файла по MIME-типу, с запасным вариантом — по имени.
fn image_extension(content_type: Option<&str>, file_name: &str) -> Option<String> {
    if let Some(content_type) = content_type {
        let essence = content_type
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        match essence.as_str() {
            "image/jpeg" | "image/jpg" => return Some("jpg".to_string()),
            "image/png" => return Some("png".to_string()),
            "image/webp" => return Some("webp".to_string()),
            _ => {}
        }
    }

    let extension = Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())?
        .to_ascii_lowercase();
    match extension.as_str() {
        "jpg" | "jpeg" => Some("jpg".to_string()),
        "png" => Some("png".to_string()),
        "webp" => Some("webp".to_string()),
        _ => None,
    }
}

/// Приводит строку к безопасному имени каталога: ASCII-буквы, цифры, дефисы.
fn slugify(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut previous_dash = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            previous_dash = false;
        } else if !out.is_empty() && !previous_dash {
            out.push('-');
            previous_dash = true;
        }
    }

    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "car".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Имя файла делаем уникальным по времени: одинаковые имена не перетрут друг друга.
fn image_stamp() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

/// Удаляет файл фото с диска (best-effort: сбой лишь логируем).
async fn remove_image_file(state: &AppState, url: &str) {
    let Some(relative) = url.strip_prefix(IMAGES_PREFIX) else {
        return;
    };
    match safe_join(&state.config.images_dir, relative) {
        Some(path) => {
            if let Err(error) = tokio::fs::remove_file(&path).await {
                if error.kind() != std::io::ErrorKind::NotFound {
                    tracing::warn!(error = %error, path = %path.display(), "Не удалось удалить файл фото");
                }
            }
        }
        None => tracing::warn!(url = %url, "Подозрительный путь фото — файл не удалён"),
    }
}

/// Удаляет каталог фото авто, если он опустел.
async fn remove_empty_dir(state: &AppState, urls: &[String]) {
    // Все фото одного авто лежат в одном каталоге — берём его из первого пути.
    let Some(relative) = urls.first().and_then(|url| url.strip_prefix(IMAGES_PREFIX)) else {
        return;
    };
    let Some(parent) = Path::new(relative).parent() else {
        return;
    };
    if parent.as_os_str().is_empty() {
        return;
    }
    let Some(path) = safe_join(&state.config.images_dir, &parent.to_string_lossy()) else {
        return;
    };

    // remove_dir сработает только если каталог пуст: иначе оставляем как есть.
    if let Err(error) = tokio::fs::remove_dir(&path).await {
        if error.kind() != std::io::ErrorKind::NotFound {
            tracing::debug!(error = %error, path = %path.display(), "Каталог фото не удалён");
        }
    }
}

/// Присоединяет относительный путь к базовому, отвергая `..` и абсолютные пути.
///
/// Это защита от выхода за пределы каталога фото: даже если в БД (или от
/// клиента) попадёт путь вида `../../etc/passwd`, мы его не применим.
fn safe_join(base: &str, relative: &str) -> Option<PathBuf> {
    let mut path = PathBuf::from(base);
    for component in Path::new(relative).components() {
        match component {
            Component::Normal(part) => path.push(part),
            Component::CurDir => {}
            // Корень, `..` и префиксы (Windows) запрещены.
            _ => return None,
        }
    }
    Some(path)
}
