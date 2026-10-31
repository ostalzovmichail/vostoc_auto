//! GET-страницы веб-админки и htmx-фрагменты для них.

use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;

use crate::api::admin::forms::CarForm;
use crate::api::admin::templates::{
    CarDetailPageTemplate, CarFormPageTemplate, CarsPageTemplate, CarsTableTemplate,
    DashboardTemplate, ErrorPageTemplate, VisitsCardTemplate,
};
use crate::api::AppState;
use crate::core::models::{AdminCarFilter, Car, Stats, VisitStats, CAR_STATUSES};

/// Сколько автомобилей показывать на одной странице списка.
const PAGE_SIZE: i64 = 20;

/// Параметры списка автомобилей из query string.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CarsQuery {
    pub status: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub page: Option<i64>,
}

/// Готовые данные для страницы/фрагмента списка.
pub struct CarsView {
    pub cars: Vec<Car>,
    pub stats: Stats,
    pub status: String,
    pub query: String,
    pub sort: String,
    pub page: i64,
    pub pages: i64,
    pub total: i64,
}

/// GET /admin/ — дашборд.
pub async fn dashboard(State(state): State<AppState>) -> Response {
    let stats = match state.db.cars_stats().await {
        Ok(stats) => stats,
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить статистику для админки");
            return error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Ошибка базы данных",
                "Не удалось получить статистику каталога.",
            );
        }
    };

    let visits = state.db.site_visits().await.unwrap_or(VisitStats {
        total: 0,
        today: 0,
        last_7_days: 0,
    });
    let avg_days = state.db.avg_deal_duration_days().await.ok().flatten();

    let latest_filter = AdminCarFilter {
        limit: 5,
        offset: 0,
        ..AdminCarFilter::default()
    };
    let latest = state
        .db
        .admin_filter_cars(&latest_filter)
        .await
        .unwrap_or_default();

    render(&DashboardTemplate {
        page_title: "Обзор",
        nav: "dashboard",
        stats,
        visits,
        avg_days,
        latest,
    })
}

/// GET /admin/cars — страница списка.
pub async fn cars_page(State(state): State<AppState>, Query(query): Query<CarsQuery>) -> Response {
    match load_cars(&state, &query).await {
        Ok(view) => render(&CarsPageTemplate {
            page_title: "Автомобили",
            nav: "cars",
            cars: view.cars,
            stats: view.stats,
            status: view.status,
            query: view.query,
            sort: view.sort,
            page: view.page,
            pages: view.pages,
            total: view.total,
        }),
        Err(message) => error_page(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Ошибка каталога",
            &message,
        ),
    }
}

/// GET /admin/partials/cars — обновление таблицы без перезагрузки.
pub async fn cars_partial(
    State(state): State<AppState>,
    Query(query): Query<CarsQuery>,
) -> Response {
    match load_cars(&state, &query).await {
        Ok(view) => render(&CarsTableTemplate {
            cars: view.cars,
            page: view.page,
            pages: view.pages,
            total: view.total,
        }),
        Err(message) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(format!(
                "<div class=\"a-alert a-alert-error\">{message}</div>"
            )),
        )
            .into_response(),
    }
}

/// GET /admin/partials/visits — карточка посещений.
pub async fn visits_partial(State(state): State<AppState>) -> Response {
    match state.db.site_visits().await {
        Ok(visits) => render(&VisitsCardTemplate { visits }),
        Err(error) => {
            tracing::warn!(error = %error, "Не удалось получить посещения");
            error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Ошибка счётчика",
                "Не удалось получить счётчик посещений.",
            )
        }
    }
}

/// GET /admin/cars/new — форма создания.
pub async fn car_new_page(State(state): State<AppState>) -> Response {
    let _ = state; // форма не требует данных из БД
    render(&CarFormPageTemplate {
        page_title: "Новый автомобиль",
        nav: "new",
        form: CarForm::with_defaults(),
        errors: Vec::new(),
        is_edit: false,
        action: "/admin/cars".to_string(),
        car_id: None,
    })
}

/// GET /admin/cars/:id — карточка автомобиля.
pub async fn car_detail_page(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    match state.db.car_by_id(id).await {
        Ok(Some(car)) => render(&CarDetailPageTemplate {
            page_title: "Карточка авто",
            nav: "cars",
            car,
        }),
        Ok(None) => error_page(
            StatusCode::NOT_FOUND,
            "Автомобиль не найден",
            "Такой записи нет в каталоге.",
        ),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить авто для админки");
            error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Ошибка базы данных",
                "Не удалось загрузить автомобиль.",
            )
        }
    }
}

/// GET /admin/cars/:id/edit — форма редактирования.
pub async fn car_edit_page(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    match state.db.car_by_id(id).await {
        Ok(Some(car)) => render(&CarFormPageTemplate {
            page_title: "Редактирование",
            nav: "cars",
            form: CarForm::from_car(&car),
            errors: Vec::new(),
            is_edit: true,
            action: format!("/admin/cars/{id}"),
            car_id: Some(id),
        }),
        Ok(None) => error_page(
            StatusCode::NOT_FOUND,
            "Автомобиль не найден",
            "Такой записи нет в каталоге.",
        ),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить авто для редактирования");
            error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Ошибка базы данных",
                "Не удалось загрузить автомобиль.",
            )
        }
    }
}

/// Загружает список и счётчики по параметрам запроса.
pub async fn load_cars(state: &AppState, query: &CarsQuery) -> Result<CarsView, String> {
    let status = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|value| CAR_STATUSES.contains(value))
        .unwrap_or_default()
        .to_string();
    let search = query.q.as_deref().unwrap_or_default().trim().to_string();
    let sort = normalize_sort(query.sort.as_deref());
    let requested_page = query.page.unwrap_or(1).max(1);

    let filter = AdminCarFilter {
        status: if status.is_empty() {
            None
        } else {
            Some(status.clone())
        },
        query: if search.is_empty() {
            None
        } else {
            Some(search.clone())
        },
        sort: Some(sort.clone()),
        limit: PAGE_SIZE,
        offset: 0,
    };

    let total = state.db.admin_cars_total(&filter).await.map_err(|error| {
        tracing::error!(error = %error, "Не удалось посчитать автомобили");
        "Не удалось загрузить список автомобилей.".to_string()
    })?;

    let pages = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let page = requested_page.min(pages);
    let filter = AdminCarFilter {
        offset: (page - 1) * PAGE_SIZE,
        ..filter
    };

    let cars = state.db.admin_filter_cars(&filter).await.map_err(|error| {
        tracing::error!(error = %error, "Не удалось получить автомобили");
        "Не удалось загрузить список автомобилей.".to_string()
    })?;

    // Счётчики статусов не зависят от поиска — они показывают размеры групп.
    let stats = state.db.cars_stats().await.unwrap_or(Stats {
        total: 0,
        in_stock: 0,
        in_transit: 0,
        on_order: 0,
        sold: 0,
    });

    Ok(CarsView {
        cars,
        stats,
        status,
        query: search,
        sort,
        page,
        pages,
        total,
    })
}

/// Оставляет только поддерживаемые значения сортировки.
fn normalize_sort(sort: Option<&str>) -> String {
    match sort {
        Some("price-asc") => "price-asc",
        Some("price-desc") => "price-desc",
        Some("mileage") => "mileage",
        Some("sold") => "sold",
        _ => "new",
    }
    .to_string()
}

/// Рендерит шаблон, превращая сбой рендеринга в аккуратный ответ.
pub fn render<T: Template>(template: &T) -> Response {
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось отрисовать шаблон админки");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<div class=\"a-alert a-alert-error\">Ошибка шаблона</div>".to_string()),
            )
                .into_response()
        }
    }
}

/// Страница ошибки админки.
pub fn error_page(status: StatusCode, title: &str, message: &str) -> Response {
    let template = ErrorPageTemplate {
        page_title: title.to_string(),
        nav: "",
        message: message.to_string(),
    };
    match template.render() {
        Ok(html) => (status, Html(html)).into_response(),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось отрисовать страницу ошибки");
            (status, message.to_string()).into_response()
        }
    }
}
