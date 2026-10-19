//! Обработчики HTML-фрагментов для страницы на htmx.
//!
//! Отличие от [`crate::api::handlers::cars`]: там JSON для клиентов и Swagger,
//! здесь — готовый HTML, который htmx вставляет в разметку. Поэтому ошибки
//! тоже отдаются разметкой (иначе в блоке мелькнёт сырой JSON).

use crate::api::handlers::cars::FILTER_STATUSES;
use crate::api::templates::{
    CatalogTemplate, ContactsTemplate, HeroTemplate, SoldTemplate, StatsTemplate,
};
use crate::api::AppState;
use crate::core::models::{now_vlat_display, CarFilter, Stats};
use askama::Template;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;

/// Сколько карточек отдавать в каталоге на одной странице.
const CATALOG_LIMIT: i64 = 100;
/// Сколько сделок показывать в архиве проданных.
const SOLD_LIMIT: i64 = 50;

/// Пустая статистика — используется, если счётчики не удалось получить,
/// но сам фрагмент отрисовать можно.
fn empty_stats() -> Stats {
    Stats {
        total: 0,
        in_stock: 0,
        in_transit: 0,
        on_order: 0,
        sold: 0,
    }
}

/// HTML-заглушка ошибки: её увидит пользователь после подстановки htmx.
fn error_fragment(status: StatusCode, message: &str) -> Response {
    (
        status,
        Html(format!("<div class=\"load-error\">⚠ {message}</div>")),
    )
        .into_response()
}

/// Рендерит шаблон, превращая сбой рендеринга в аккуратный фрагмент с ошибкой.
fn render<T: Template>(template: &T) -> Response {
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось отрендерить HTML-фрагмент");
            error_fragment(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Не удалось отрисовать фрагмент",
            )
        }
    }
}

/// Телефон из контактов в формате `tel:` (пустая строка, если контактов нет).
async fn phone_tel(state: &AppState) -> String {
    state
        .db
        .contacts()
        .await
        .ok()
        .flatten()
        .map(|contact| contact.phone_primary_tel())
        .unwrap_or_default()
}

/// GET /partials/hero — главный акцент с последним авто.
pub async fn hero_partial(State(state): State<AppState>) -> Response {
    let car = match state.db.latest_car().await {
        Ok(Some(car)) => car,
        Ok(None) => return error_fragment(StatusCode::NOT_FOUND, "В базе пока нет автомобилей"),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить последнее авто для главной");
            return error_fragment(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Сервис временно недоступен",
            );
        }
    };

    let total = match state.db.cars_stats().await {
        Ok(stats) => stats.total,
        Err(error) => {
            tracing::warn!(error = %error, "Не удалось получить статистику для главной");
            0
        }
    };

    render(&HeroTemplate {
        car,
        total,
        updated: now_vlat_display(),
        phone_tel: phone_tel(&state).await,
    })
}

/// Параметры фрагмента каталога (приходят из формы на странице).
#[derive(Debug, Deserialize)]
pub struct CatalogQuery {
    pub status: Option<String>,
    pub brand: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
}

/// GET /partials/catalog — карточки каталога по фильтрам формы.
pub async fn catalog_partial(
    State(state): State<AppState>,
    Query(query): Query<CatalogQuery>,
) -> Response {
    // Незнакомый статус молча игнорируем: страница не должна показывать
    // ошибку из-за пустого значения радио-кнопки или устаревшей ссылки.
    let status = query
        .status
        .as_deref()
        .filter(|value| FILTER_STATUSES.contains(value))
        .map(str::to_string);

    let filter = CarFilter {
        status,
        brand: query.brand,
        query: query.q,
        sort: query.sort,
        limit: CATALOG_LIMIT,
        offset: 0,
    };

    let cars = match state.db.filter_cars(&filter).await {
        Ok(cars) => cars,
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить каталог");
            return error_fragment(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Каталог временно недоступен",
            );
        }
    };

    // Счётчики фильтров берём из общей статистики: они показывают размеры
    // статусных групп целиком, а не результат текущего поиска.
    let stats = match state.db.cars_stats().await {
        Ok(stats) => stats,
        Err(error) => {
            tracing::warn!(error = %error, "Не удалось получить счётчики каталога");
            empty_stats()
        }
    };

    render(&CatalogTemplate {
        cars,
        stats,
        phone_tel: phone_tel(&state).await,
    })
}

/// GET /partials/sold — архив недавно проданных.
pub async fn sold_partial(State(state): State<AppState>) -> Response {
    let cars = match state.db.sold_cars(SOLD_LIMIT).await {
        Ok(cars) => cars,
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить проданные авто");
            return error_fragment(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Архив сделок временно недоступен",
            );
        }
    };

    render(&SoldTemplate { cars })
}

/// GET /partials/stats — счётчики для блока статистики.
pub async fn stats_partial(State(state): State<AppState>) -> Response {
    let stats = match state.db.cars_stats().await {
        Ok(stats) => stats,
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить статистику");
            return error_fragment(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Статистика временно недоступна",
            );
        }
    };

    let avg_days = state
        .db
        .avg_deal_duration_days()
        .await
        .ok()
        .flatten()
        .unwrap_or(0);

    render(&StatsTemplate {
        total: stats.total,
        sold: stats.sold,
        avg_days,
    })
}

/// GET /partials/contacts — контакты, мессенджеры и форма звонка.
pub async fn contacts_partial(State(state): State<AppState>) -> Response {
    let contact = match state.db.contacts().await {
        Ok(Some(contact)) => contact,
        Ok(None) => return error_fragment(StatusCode::NOT_FOUND, "Контакты не заполнены"),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить контакты");
            return error_fragment(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Контакты временно недоступны",
            );
        }
    };

    render(&ContactsTemplate { contact })
}
