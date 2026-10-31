//! Askama-шаблоны веб-админки.
//!
//! Один шаблон — один блок страницы. Полные страницы наследуют
//! `admin/layout.html`, фрагменты для htmx лежат в `admin/partials/` и
//! рендерятся теми же структурами, что и страницы.

use askama::Template;

use crate::api::admin::forms::{CarForm, FieldError};
use crate::core::models::{Car, Stats, VisitStats};

/// Страница входа. Не наследует layout: у неё нет навигации и выхода.
#[derive(Template)]
#[template(path = "admin/login.html")]
pub struct LoginTemplate {
    pub admin_enabled: bool,
    pub error: Option<String>,
    pub token_value: String,
}

/// Дашборд: счётчики, посещения, последние авто.
#[derive(Template)]
#[template(path = "admin/dashboard.html")]
pub struct DashboardTemplate {
    pub page_title: &'static str,
    pub nav: &'static str,
    pub stats: Stats,
    pub visits: VisitStats,
    pub avg_days: Option<i64>,
    pub latest: Vec<Car>,
}

/// Страница списка автомобилей.
#[derive(Template)]
#[template(path = "admin/cars.html")]
pub struct CarsPageTemplate {
    pub page_title: &'static str,
    pub nav: &'static str,
    pub cars: Vec<Car>,
    pub stats: Stats,
    pub status: String,
    pub query: String,
    pub sort: String,
    pub page: i64,
    pub pages: i64,
    pub total: i64,
}

/// Фрагмент списка: строки таблицы и пагинация.
#[derive(Template)]
#[template(path = "admin/partials/cars_table.html")]
pub struct CarsTableTemplate {
    pub cars: Vec<Car>,
    pub page: i64,
    pub pages: i64,
    pub total: i64,
}

/// Страница создания/редактирования автомобиля.
#[derive(Template)]
#[template(path = "admin/car_form_page.html")]
pub struct CarFormPageTemplate {
    pub page_title: &'static str,
    pub nav: &'static str,
    pub form: CarForm,
    pub errors: Vec<FieldError>,
    pub is_edit: bool,
    pub action: String,
    pub car_id: Option<i64>,
}

/// Фрагмент формы (перерисовывается при ошибках валидации).
#[derive(Template)]
#[template(path = "admin/partials/car_form.html")]
pub struct CarFormFragmentTemplate {
    pub form: CarForm,
    pub errors: Vec<FieldError>,
    pub is_edit: bool,
    pub action: String,
    pub car_id: Option<i64>,
}

/// Страница автомобиля: характеристики, статус, фотографии.
#[derive(Template)]
#[template(path = "admin/car_detail.html")]
pub struct CarDetailPageTemplate {
    pub page_title: &'static str,
    pub nav: &'static str,
    pub car: Car,
}

/// Карточка статуса (фрагмент и часть страницы).
#[derive(Template)]
#[template(path = "admin/partials/status_card.html")]
pub struct StatusCardTemplate {
    pub car: Car,
}

/// Галерея фотографий (фрагмент и часть страницы).
#[derive(Template)]
#[template(path = "admin/partials/photos_grid.html")]
pub struct PhotosGridTemplate {
    pub car: Car,
}

/// Карточка посещений (дашборд и htmx-обновление).
#[derive(Template)]
#[template(path = "admin/partials/visits_card.html")]
pub struct VisitsCardTemplate {
    pub visits: VisitStats,
}

/// Всплывающее уведомление, приезжает через `hx-swap-oob`.
#[derive(Template)]
#[template(path = "admin/partials/toast.html")]
pub struct ToastTemplate {
    pub kind: &'static str,
    pub message: String,
}

/// Панель сгенерированного AI-описания.
#[derive(Template)]
#[template(path = "admin/partials/ai_description.html")]
pub struct AiDescriptionTemplate {
    pub text: Option<String>,
    pub error: Option<String>,
    pub model: String,
}

/// Простая страница ошибки (например, БД недоступна).
#[derive(Template)]
#[template(path = "admin/error.html")]
pub struct ErrorPageTemplate {
    pub page_title: String,
    pub nav: &'static str,
    pub message: String,
}
