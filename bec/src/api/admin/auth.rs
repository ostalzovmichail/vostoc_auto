//! Вход в веб-админку, cookie-сессия и защита маршрутов `/admin/*`.

use std::net::SocketAddr;

use askama::Template;
use axum::body::Body;
use axum::extract::{ConnectInfo, Form, Request, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Redirect, Response};

use crate::api::admin::forms::LoginForm;
use crate::api::admin::session;
use crate::api::admin::templates::LoginTemplate;
use crate::api::AppState;

/// Заголовок htmx, означающий «ответ нужно вставить без перезагрузки страницы».
const HX_REQUEST: &str = "hx-request";
/// Заголовок, которым htmx выполняет клиентский редирект.
const HX_REDIRECT: &str = "hx-redirect";

/// GET /admin/login — форма входа.
pub async fn login_page(State(state): State<AppState>) -> Response {
    render_login(&state, None, "")
}

/// POST /admin/login — проверка токена и выдача cookie-сессии.
pub async fn login_submit(
    State(state): State<AppState>,
    headers: HeaderMap,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Form(form): Form<LoginForm>,
) -> Response {
    let Some(expected) = state.config.admin_token.as_deref() else {
        tracing::error!("Попытка входа: ADMIN_TOKEN не задан");
        return render_login(
            &state,
            Some("Админка отключена: на сервере не задан ADMIN_TOKEN".to_string()),
            "",
        );
    };

    let key = client_key(&headers, connect_info.as_ref());
    if state.login_limiter.is_blocked(&key) {
        tracing::warn!(%key, "Слишком много неудачных попыток входа");
        return render_login(
            &state,
            Some("Слишком много неудачных попыток. Попробуйте через 10 минут.".to_string()),
            "",
        );
    }

    if !constant_time_eq(form.token.trim().as_bytes(), expected.as_bytes()) {
        state.login_limiter.record_failure(&key);
        tracing::warn!(%key, "Неверный токен администратора");
        return render_login(
            &state,
            Some("Неверный токен администратора".to_string()),
            "",
        );
    }

    state.login_limiter.reset(&key);
    let (value, _) = session::issue(expected, session::now_unix());
    tracing::info!(%key, "Администратор вошёл в веб-панель");

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin/")
        .header(
            header::SET_COOKIE,
            session::set_cookie(&value, state.config.admin_cookie_secure),
        )
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// POST /admin/logout — удаление cookie и возврат на форму входа.
pub async fn logout(State(state): State<AppState>) -> Response {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin/login")
        .header(
            header::SET_COOKIE,
            session::clear_cookie(state.config.admin_cookie_secure),
        )
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// Middleware защищённых страниц `/admin/*`.
///
/// Проверяет подпись cookie и (для изменяющих запросов) same-origin.
pub async fn require_admin_page(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let Some(secret) = state.config.admin_token.as_deref() else {
        return redirect_or_htmx(&request, "/admin/login");
    };

    let valid = session::cookie_value(request.headers())
        .is_some_and(|value| session::verify(secret, &value, session::now_unix()));

    if !valid {
        tracing::warn!(path = %request.uri().path(), "Отказ: нет валидной админской сессии");
        return redirect_or_htmx(&request, "/admin/login");
    }

    if !matches!(request.method().as_str(), "GET" | "HEAD" | "OPTIONS") && !same_origin(&request) {
        tracing::warn!(path = %request.uri().path(), "Отказ: изменяющий запрос с чужого origin");
        return (StatusCode::FORBIDDEN, "Запрос отклонён: неверный origin").into_response();
    }

    next.run(request).await
}

/// Рендерит страницу входа.
fn render_login(state: &AppState, error: Option<String>, token_value: &str) -> Response {
    let template = LoginTemplate {
        admin_enabled: state.config.admin_enabled(),
        error,
        token_value: token_value.to_string(),
    };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось отрисовать страницу входа");
            (StatusCode::INTERNAL_SERVER_ERROR, "Ошибка шаблона").into_response()
        }
    }
}

/// Редирект для обычного запроса и `HX-Redirect` для htmx.
pub fn redirect_or_htmx(request: &Request, to: &str) -> Response {
    if is_htmx(request.headers()) {
        Response::builder()
            .status(StatusCode::OK)
            .header(HX_REDIRECT, to)
            .body(Body::empty())
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
    } else {
        Redirect::to(to).into_response()
    }
}

/// Признак htmx-запроса.
pub fn is_htmx(headers: &HeaderMap) -> bool {
    headers
        .get(HX_REQUEST)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

/// Проверяет, что изменяющий запрос пришёл с того же origin.
///
/// `SameSite=Strict` уже не даёт браузеру отправить cookie с чужого сайта,
/// но дополнительная проверка `Origin`/`Sec-Fetch-Site` закрывает старые
/// браузеры и нестандартные клиенты.
fn same_origin(request: &Request) -> bool {
    let headers = request.headers();

    if let Some(site) = headers
        .get("sec-fetch-site")
        .and_then(|value| value.to_str().ok())
    {
        if !matches!(site, "same-origin" | "none") {
            return false;
        }
    }

    let Some(origin) = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
    else {
        // Origin может отсутствовать у не-браузерных клиентов; cookie-сессия
        // всё равно проверена, поэтому такой запрос не опаснее curl с токеном.
        return true;
    };

    // За обратным прокси Host может быть внутренним адресом, а публичный
    // домен приходить в X-Forwarded-Host. Принимаем любой из них.
    let hosts = [
        headers
            .get(header::HOST)
            .and_then(|value| value.to_str().ok()),
        headers
            .get("x-forwarded-host")
            .and_then(|value| value.to_str().ok()),
    ];

    if hosts.iter().all(Option::is_none) {
        return false;
    }

    let authority = origin
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(origin);
    hosts
        .into_iter()
        .flatten()
        .filter_map(|host| host.split(',').next())
        .any(|host| same_host(authority, host.trim()))
}

/// Сравнение с учётом того, что браузер может опускать порт по умолчанию.
fn same_host(authority: &str, host: &str) -> bool {
    authority == host
        || authority.strip_suffix(":443") == Some(host)
        || authority.strip_suffix(":80") == Some(host)
}

/// Ключ для лимитера: IP из `X-Forwarded-For` или адрес TCP-соединения.
fn client_key(headers: &HeaderMap, connect_info: Option<&ConnectInfo<SocketAddr>>) -> String {
    if let Some(forwarded) = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return forwarded.to_string();
    }

    connect_info
        .map(|info| info.0.ip().to_string())
        .unwrap_or_else(|| "local".to_string())
}

/// Сравнение токена за постоянное время.
fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for (a, b) in left.iter().zip(right) {
        difference |= a ^ b;
    }
    difference == 0
}
