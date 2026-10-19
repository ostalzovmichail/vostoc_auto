use crate::api::models::{bad_request, internal_error, not_found, ApiResult, Car, Contact, Stats};
use crate::api::AppState;
use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

/// Статусы, допустимые в фильтре `GET /api/cars`.
///
/// `sold` сюда не входит: проданные авто отдаёт отдельный эндпоинт
/// `/api/cars/sold`, и смешивать их с активным каталогом не нужно.
pub(crate) const FILTER_STATUSES: [&str; 3] = ["stock", "transit", "order"];

/// Значение по умолчанию для `limit`, если параметр не передан.
const DEFAULT_LIMIT: i64 = 50;
/// Верхняя граница `limit`: защищает БД и клиента от выдачи всей таблицы.
const MAX_LIMIT: i64 = 100;

/// Параметры запроса `GET /api/cars`.
#[derive(Debug, Deserialize)]
pub struct CarsQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Ограничивает `limit` разумными рамками, а отрицательный `offset` — нулём.
pub(crate) fn normalize_paging(query: &CarsQuery) -> (i64, i64) {
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = query.offset.unwrap_or(0).max(0);
    (limit, offset)
}

/// GET /api/cars/latest — последнее добавленное авто.
#[utoipa::path(
    get,
    path = "/api/cars/latest",
    responses(
        (status = 200, description = "Последнее добавленное авто", body = Car),
        (status = 404, description = "База пуста", body = ApiResponse),
        (status = 500, description = "Ошибка БД", body = ApiResponse)
    ),
    tag = "Cars"
)]
pub async fn latest_car_handler(State(state): State<AppState>) -> Result<Json<Car>, ApiResult> {
    match state.db.latest_car().await {
        Ok(Some(car)) => Ok(Json(car)),
        Ok(None) => Err(not_found("В базе пока нет автомобилей")),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить последнее авто");
            Err(internal_error())
        }
    }
}

/// GET /api/cars — автомобили в продаже (без проданных).
///
/// Фильтр `status` принимает только `stock`, `transit` или `order`.
#[utoipa::path(
    get,
    path = "/api/cars",
    params(
        ("status" = Option<String>, Query, description = "Фильтр: stock, transit, order"),
        ("limit" = Option<i64>, Query, description = "Размер страницы, 1..=100 (по умолчанию 50)"),
        ("offset" = Option<i64>, Query, description = "Смещение выборки (по умолчанию 0)")
    ),
    responses(
        (status = 200, description = "Список автомобилей", body = [Car]),
        (status = 400, description = "Недопустимый статус", body = ApiResponse),
        (status = 500, description = "Ошибка БД", body = ApiResponse)
    ),
    tag = "Cars"
)]
pub async fn cars_handler(
    State(state): State<AppState>,
    Query(query): Query<CarsQuery>,
) -> Result<Json<Vec<Car>>, ApiResult> {
    let status = match query.status.as_deref() {
        None => None,
        Some(status) if FILTER_STATUSES.contains(&status) => Some(status),
        Some(other) => {
            // Значение приходит от клиента: логируем как нештатное, но данные
            // в ответ не подставляем, чтобы не отражать ввод обратно.
            tracing::warn!(status = %other, "Недопустимое значение фильтра status");
            return Err(bad_request(
                "Недопустимый статус. Разрешены: stock, transit, order",
            ));
        }
    };

    let (limit, offset) = normalize_paging(&query);

    match state.db.list_cars(status, limit, offset).await {
        Ok(cars) => Ok(Json(cars)),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить список авто");
            Err(internal_error())
        }
    }
}

/// GET /api/cars/sold — проданные автомобили.
#[utoipa::path(
    get,
    path = "/api/cars/sold",
    params(
        ("limit" = Option<i64>, Query, description = "Сколько записей вернуть, 1..=100 (по умолчанию 50)")
    ),
    responses(
        (status = 200, description = "Список проданных авто", body = [Car]),
        (status = 500, description = "Ошибка БД", body = ApiResponse)
    ),
    tag = "Cars"
)]
pub async fn sold_cars_handler(
    State(state): State<AppState>,
    Query(query): Query<CarsQuery>,
) -> Result<Json<Vec<Car>>, ApiResult> {
    let (limit, _) = normalize_paging(&query);

    match state.db.sold_cars(limit).await {
        Ok(cars) => Ok(Json(cars)),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить список проданных авто");
            Err(internal_error())
        }
    }
}

/// GET /api/stats — статистика базы.
#[utoipa::path(
    get,
    path = "/api/stats",
    responses(
        (status = 200, description = "Статистика по базе", body = Stats),
        (status = 500, description = "Ошибка БД", body = ApiResponse)
    ),
    tag = "Cars"
)]
pub async fn stats_handler(State(state): State<AppState>) -> Result<Json<Stats>, ApiResult> {
    match state.db.cars_stats().await {
        Ok(stats) => Ok(Json(stats)),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить статистику");
            Err(internal_error())
        }
    }
}

/// GET /api/contacts — контактная информация.
#[utoipa::path(
    get,
    path = "/api/contacts",
    responses(
        (status = 200, description = "Контактная информация", body = Contact),
        (status = 404, description = "Контакты не заданы", body = ApiResponse),
        (status = 500, description = "Ошибка БД", body = ApiResponse)
    ),
    tag = "Cars"
)]
pub async fn contacts_handler(State(state): State<AppState>) -> Result<Json<Contact>, ApiResult> {
    match state.db.contacts().await {
        Ok(Some(contact)) => Ok(Json(contact)),
        Ok(None) => Err(not_found("Контактная информация не заполнена")),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить контакты");
            Err(internal_error())
        }
    }
}
