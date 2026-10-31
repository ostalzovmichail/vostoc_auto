//! Веб-админка: страницы на askama + htmx и JSON API `/api/admin/*`.
//!
//! Слои те же, что и у публичного сайта: обработчики → `AppState` → `core`.
//! HTML-маршруты и JSON API используют общие функции БД и валидации, но не
//! вызывают друг друга по HTTP.

pub mod actions;
pub mod api;
pub mod auth;
pub mod forms;
pub mod media;
pub mod pages;
pub mod routes;
pub mod session;
pub mod templates;
