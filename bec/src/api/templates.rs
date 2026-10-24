//! Шаблоны HTML-фрагментов для страницы на htmx.
//!
//! Один шаблон — один блок страницы. Шаблонизатор askama компилирует разметку
//! вместе с крейтом, поэтому структура проверяется на этапе сборки, а данные
//! приходят из тех же моделей [`crate::core::models`], что и JSON API.

use crate::core::models::{Car, Contact, Stats};
use askama::Template;

/// Главный акцент: последнее добавленное авто, галерея, спецификация и цена.
#[derive(Template)]
#[template(path = "partials/hero.html")]
pub struct HeroTemplate {
    pub car: Car,
    /// Всего автомобилей в базе (строка под блоком).
    pub total: i64,
    /// Время обновления базы по Владивостоку.
    pub updated: String,
    /// Телефон для ссылок `tel:`.
    pub phone_tel: String,
}

/// Каталог автомобилей: карточки + актуальные счётчики для фильтров.
#[derive(Template)]
#[template(path = "partials/catalog.html")]
pub struct CatalogTemplate {
    pub cars: Vec<Car>,
    pub stats: Stats,
}

/// Детальная информация об автомобиле (модальное окно).
#[derive(Template)]
#[template(path = "partials/car_detail.html")]
pub struct CarDetailTemplate {
    pub car: Car,
    pub phone_tel: String,
}

/// Архив недавно проданных автомобилей.
#[derive(Template)]
#[template(path = "partials/sold.html")]
pub struct SoldTemplate {
    pub cars: Vec<Car>,
}

/// Сводная статистика базы с анимированными счётчиками.
#[derive(Template)]
#[template(path = "partials/stats.html")]
pub struct StatsTemplate {
    pub total: i64,
    pub sold: i64,
    /// Средний срок сделки в днях.
    pub avg_days: i64,
}

/// Контакты автосалона и форма обратного звонка.
#[derive(Template)]
#[template(path = "partials/contacts.html")]
pub struct ContactsTemplate {
    pub contact: Contact,
}
