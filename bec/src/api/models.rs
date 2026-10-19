use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

// Модели данных (Car/Stats/Contact) объявлены в `core::models`: их использует
// трейт `Database`, а значит слой core не должен зависеть от api. Отсюда лишь
// реэкспорт — обработчики и OpenAPI-схема по-прежнему берут типы из `api::models`.
pub use crate::core::models::{Car, Contact, NewCar, Stats, StatusUpdate, VisitStats};

/// Тип ответа любого эндпоинта: код HTTP + JSON-тело.
/// Алиас избавляет обработчики от повторяющегося кортежа `(StatusCode, Json<...>)`.
pub type ApiResult = (StatusCode, Json<ApiResponse>);

/// Стандартное тело ответа сервера.
#[derive(Serialize, ToSchema)]
pub struct ApiResponse {
    /// Машиночитаемый статус: "success" или "error"
    pub status: ResponseStatus,
    /// Человекочитаемое сообщение
    pub message: String,
}

/// Статус в теле ответа.
///
/// Enum вместо строки: компилятор не даст вернуть произвольный текст,
/// а serde сериализует варианты ровно как раньше — "success"/"error".
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Success,
    Error,
}

impl ApiResponse {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            status: ResponseStatus::Success,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: ResponseStatus::Error,
            message: message.into(),
        }
    }
}

/// Успешный ответ: HTTP 200 + тело со статусом `success`.
pub fn ok(message: impl Into<String>) -> ApiResult {
    (StatusCode::OK, Json(ApiResponse::success(message)))
}

/// Ответ на внутреннюю ошибку сервера.
///
/// Клиенту отдаём только общее сообщение. Тексты ошибок БД, пути и SQL
/// уходят в лог (см. обработчики) — раскрывать их наружу нельзя:
/// это помогает злоумышленнику и учитывает детали инфраструктуры.
pub fn internal_error() -> ApiResult {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiResponse::error("Внутренняя ошибка сервера")),
    )
}

/// Ответ 404: запрошенных данных в базе нет.
pub fn not_found(message: impl Into<String>) -> ApiResult {
    (StatusCode::NOT_FOUND, Json(ApiResponse::error(message)))
}

/// Ответ 400: клиент передал некорректные параметры запроса.
pub fn bad_request(message: impl Into<String>) -> ApiResult {
    (StatusCode::BAD_REQUEST, Json(ApiResponse::error(message)))
}

/// Ответ 401: не передан или неверен токен администратора.
pub fn unauthorized(message: impl Into<String>) -> ApiResult {
    (StatusCode::UNAUTHORIZED, Json(ApiResponse::error(message)))
}

/// Ответ 503: функциональность не настроена на сервере (например, нет токена).
pub fn service_unavailable(message: impl Into<String>) -> ApiResult {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ApiResponse::error(message)),
    )
}

/// Ответ 502: внешний сервис (OpenRouter) недоступен или вернул ошибку.
///
/// Как и в [`internal_error`], наружу уходит только человекочитаемое
/// сообщение — детали upstream-ответа логируются в обработчике.
pub fn bad_gateway(message: impl Into<String>) -> ApiResult {
    (StatusCode::BAD_GATEWAY, Json(ApiResponse::error(message)))
}
