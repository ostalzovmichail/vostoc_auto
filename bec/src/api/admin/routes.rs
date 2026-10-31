//! Маршруты веб-админки.
//!
//! Публичные: вход и статика (`/admin/assets/*`). Всё остальное обёрнуто
//! проверкой cookie-сессии. JSON API `/api/admin/*` собирается отдельно
//! в `api::router` и по-прежнему требует заголовок `X-Admin-Token`.

use axum::middleware;
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::Router;
use tower_http::services::ServeDir;

use crate::api::admin::auth::{login_page, login_submit, logout, require_admin_page};
use crate::api::admin::{actions, pages};
use crate::api::AppState;

/// Каталог статики админки (CSS/JS).
const ADMIN_FRONTEND_DIR: &str = "frontend/admin";

/// Собирает роутер `/admin/*`.
pub fn router(state: AppState) -> Router<AppState> {
    let public = Router::new()
        .route("/admin", get(|| async { Redirect::to("/admin/") }))
        .route("/admin/login", get(login_page).post(login_submit))
        .nest_service("/admin/assets", ServeDir::new(ADMIN_FRONTEND_DIR));

    let protected = Router::new()
        .route("/admin/", get(pages::dashboard))
        .route("/admin/logout", post(logout))
        .route(
            "/admin/cars",
            get(pages::cars_page).post(actions::create_car),
        )
        .route("/admin/cars/new", get(pages::car_new_page))
        .route(
            "/admin/cars/:id",
            get(pages::car_detail_page).post(actions::update_car),
        )
        .route("/admin/cars/:id/edit", get(pages::car_edit_page))
        .route("/admin/cars/:id/status", post(actions::set_status))
        .route("/admin/cars/:id/delete", post(actions::delete_car))
        .route("/admin/cars/:id/images", post(actions::upload_images))
        .route("/admin/cars/:id/images/delete", post(actions::delete_image))
        .route("/admin/cars/:id/images/cover", post(actions::set_cover))
        .route("/admin/cars/:id/images/move", post(actions::move_image))
        .route("/admin/ai/description", post(actions::ai_description))
        .route("/admin/partials/cars", get(pages::cars_partial))
        .route("/admin/partials/visits", get(pages::visits_partial))
        .route_layer(middleware::from_fn_with_state(state, require_admin_page));

    public.merge(protected)
}
