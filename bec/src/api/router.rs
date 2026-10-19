use crate::api::handlers::{admin, ai, basic, cars, pages};
use crate::api::models::{
    service_unavailable, unauthorized, ApiResponse, Car, Contact, ResponseStatus, Stats,
};
use crate::api::AppState;
use crate::core::config::Config;
use axum::{
    extract::{Request, State},
    http::{header, HeaderName, HeaderValue, Method},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
    Router,
};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// OpenAPI-спецификация всего сервера.
#[derive(OpenApi)]
#[openapi(
    paths(
        basic::home_handler,
        basic::test_handler,
        basic::db_handler,
        cars::latest_car_handler,
        cars::cars_handler,
        cars::sold_cars_handler,
        cars::stats_handler,
        cars::contacts_handler
    ),
    components(
        // Схемы перечисляем явно: без этого Swagger UI покажет битую
        // ссылку на схему соответствующего поля.
        schemas(ApiResponse, ResponseStatus, Car, Stats, Contact)
    ),
    tags(
        (name = "Basic", description = "Базовые эндпоинты сервера"),
        (name = "Cars", description = "Каталог автомобилей, статистика и контакты")
    )
)]
struct ApiDoc;

/// Собирает роутер сервера: маршруты + (в dev) Swagger + слои CORS и трассировки.
pub fn create_router(state: AppState) -> Router {
    // Флаги читаем до перемещения состояния в роутер (`with_state` забирает его).
    let docs_enabled = state.config.docs_enabled();
    let cors = cors_layer(&state.config);
    let images_dir = state.config.images_dir.clone();

    // Админ-маршруты оборачиваем проверкой токена. `from_fn_with_state` получает
    // копию состояния, поэтому проверка работает ещё до `with_state`.
    let admin =
        admin_routes().route_layer(middleware::from_fn_with_state(state.clone(), require_admin));

    // Счётчик посещений вешаем на весь роутер: сам middleware считает только
    // запросы к публичной странице `/ui/`.
    let visit_counter = middleware::from_fn_with_state(state.clone(), count_site_visit);

    let mut router = Router::new()
        .merge(api_routes())
        .merge(admin)
        .merge(partials_routes())
        .merge(static_routes(&images_dir))
        .merge(frontend_routes());
    if docs_enabled {
        router = router.merge(docs_route());
    }

    router
        .with_state(state)
        // Слои применяются ко всем маршрутам, включая Swagger.
        .layer(cors)
        .layer(visit_counter)
        .layer(TraceLayer::new_for_http())
}

/// Маршруты API. Вынесены в отдельную функцию, чтобы `create_router`
/// читалась как последовательность настройки, а не как одна длинная цепочка.
fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(basic::home_handler))
        .route("/test", get(basic::test_handler))
        .route("/db", get(basic::db_handler))
        // Порядок регистрации не важен: axum сравнивает пути целиком,
        // и `/api/cars/latest` не перехватывается маршрутом `/api/cars`.
        .route("/api/cars", get(cars::cars_handler))
        .route("/api/cars/latest", get(cars::latest_car_handler))
        .route("/api/cars/sold", get(cars::sold_cars_handler))
        .route("/api/stats", get(cars::stats_handler))
        .route("/api/contacts", get(cars::contacts_handler))
}

/// Заголовок, в котором админ-клиент передаёт токен.
const ADMIN_TOKEN_HEADER: &str = "x-admin-token";

/// Маршруты админ-API: управление каталогом.
///
/// Все они обёрнуты проверкой токена (см. [`require_admin`]). Пути под
/// `/api/admin/*` отделены от публичного `/api/*`, чтобы случайно не открыть
/// запись данных.
fn admin_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/admin/cars",
            get(admin::list_handler).post(admin::create_handler),
        )
        .route("/api/admin/cars/:id", delete(admin::delete_handler))
        .route(
            "/api/admin/cars/:id/status",
            patch(admin::set_status_handler),
        )
        .route(
            "/api/admin/cars/:id/images",
            post(admin::upload_image_handler).delete(admin::delete_image_handler),
        )
        // Счётчик посещений сайта: числа видит только админка.
        .route("/api/admin/stats/visits", get(admin::visits_handler))
        // AI-функции: генерация текстов через OpenRouter. Тоже под токеном —
        // обращение к платному провайдеру не должно быть доступно всем.
        .route("/api/admin/ai/description", post(ai::describe_handler))
}

/// Проверяет токен администратора в заголовке `X-Admin-Token`.
///
/// Если токен не задан в конфиге — отвечаем 503. Это важнее, чем кажется:
/// «не настроено» честнее и безопаснее, чем молча открыть запись данных всем.
async fn require_admin(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let Some(expected) = state.config.admin_token.as_deref() else {
        tracing::error!("ADMIN_TOKEN не задан — запрос к админ-API отклонён");
        return service_unavailable("Админ-API отключён: на сервере не задан ADMIN_TOKEN")
            .into_response();
    };

    let provided = request
        .headers()
        .get(ADMIN_TOKEN_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    if !constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
        tracing::warn!("Отказано в доступе к админ-API: неверный или отсутствующий токен");
        return unauthorized("Неверный токен администратора").into_response();
    }

    next.run(request).await
}

/// Сравнение за постоянное время: иначе токен можно подбирать по времени ответа.
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

/// Считает просмотры публичной страницы сайта (`/ui/`).
///
/// Учитываем только успешно отданный GET: редирект `/ui` → `/ui/` и запросы
/// картинок, CSS и htmx-фрагментов в счётчик не попадают. Сбой БД не должен
/// ломать показ сайта — поэтому ошибку лишь логируем.
async fn count_site_visit(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let is_page_view =
        request.method() == Method::GET && matches!(request.uri().path(), "/ui" | "/ui/");

    let response = next.run(request).await;

    if is_page_view && response.status().is_success() {
        if let Err(error) = state.db.record_site_visit().await {
            tracing::warn!(error = %error, "Не удалось учесть посещение сайта");
        }
    }

    response
}

/// Раздаёт фотографии по префиксу `/images`.
///
/// Каталог берётся из конфига (`IMAGES_DIR`, по умолчанию `static/images`) —
/// тот же, куда пишет загрузка из админ-API: чтение и запись всегда совпадают.
/// В БД хранятся относительные пути (`/images/...`), абсолютный адрес (хост API)
/// подставляет фронтенд — так данные не привязаны к окружению.
/// CORS здесь не нужен: браузер грузит `<img>` без preflight.
fn static_routes(images_dir: &str) -> Router<AppState> {
    Router::new().nest_service("/images", ServeDir::new(images_dir))
}

/// HTML-фрагменты для страницы на htmx.
///
/// Держим их отдельно от `/api/*`: API — это контракт для машин (JSON),
/// а здесь — разметка для браузера. Смешивать форматы в одном префиксе нельзя.
fn partials_routes() -> Router<AppState> {
    Router::new()
        .route("/partials/hero", get(pages::hero_partial))
        .route("/partials/catalog", get(pages::catalog_partial))
        .route("/partials/sold", get(pages::sold_partial))
        .route("/partials/stats", get(pages::stats_partial))
        .route("/partials/contacts", get(pages::contacts_partial))
}

/// Каталог нового фронтенда (`frontend/`): HTML, CSS, htmx и скрипты.
///
/// Раздаём его сам сервер, чтобы страница и фрагменты были на одном origin:
/// htmx ходит по относительным путям, а CORS не нужен вовсе.
const FRONTEND_DIR: &str = "frontend";

/// Монтирует [`FRONTEND_DIR`] на `/ui`; `append_index_html_on_directories`
/// отдаёт `index.html` при запросе каталога (`/ui/`).
fn frontend_routes() -> Router<AppState> {
    Router::new().nest_service(
        "/ui",
        ServeDir::new(FRONTEND_DIR).append_index_html_on_directories(true),
    )
}

/// Swagger UI. Подключается только в dev (см. `Config::docs_enabled`):
/// в проде документация раскрывает поверхность API.
fn docs_route() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/swagger-ui/openapi.json", ApiDoc::openapi())
}

/// Настройка CORS в зависимости от окружения.
///
/// * dev — `permissive`: любой origin и любой метод, чтобы фронт/Swagger
///   на другом порту работали без возни.
/// * prod со списком origin'ов — отвечаем только доверенным.
/// * prod без списка — CORS-заголовки не отдаём вообще: браузеры сами
///   блокируют чужие origin'ы (безопасно по умолчанию).
fn cors_layer(config: &Config) -> CorsLayer {
    if config.environment.is_dev() {
        return CorsLayer::permissive();
    }

    if config.cors_allowed_origins.is_empty() {
        return CorsLayer::new();
    }

    let origins: Vec<HeaderValue> = config
        .cors_allowed_origins
        .iter()
        .filter_map(|raw| match raw.parse() {
            Ok(origin) => Some(origin),
            Err(_) => {
                // Некорректный origin пропускаем, но обязательно логируем:
                // молча терять настройки администратора нельзя.
                tracing::warn!(origin = %raw, "Пропущен некорректный origin в CORS_ALLOWED_ORIGINS");
                None
            }
        })
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        // Админ-приложение (в том числе Flutter Web) меняет данные, поэтому
        // методов больше, чем у публичного чтения. Главный рубеж — список
        // origin'ов выше: чужие источники по-прежнему не допускаются.
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([
            header::CONTENT_TYPE,
            HeaderName::from_static(ADMIN_TOKEN_HEADER),
        ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::{Database, SqliteDb};
    use crate::core::models::{Car, Contact, Stats};
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt; // ServiceExt::oneshot — вызов роутера без TCP

    /// Собирает тестовое приложение поверх in-memory SQLite:
    /// тесты не трогают реальные файлы БД.
    async fn test_app() -> Router {
        let db = SqliteDb::new("sqlite::memory:")
            .await
            .expect("Не удалось создать in-memory БД");
        create_router(AppState::new(db, Config::for_tests()))
    }

    #[tokio::test]
    async fn test_home_handler() {
        let response = test_app()
            .await
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_test_handler() {
        let response = test_app()
            .await
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_db_handler() {
        let response = test_app()
            .await
            .oneshot(Request::builder().uri("/db").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    /// БД, которая всегда завершается ошибкой — нужно для проверки
    /// обработки сбоев (в проде так бывает, и сервер не должен падать).
    struct FailingDb;

    #[async_trait::async_trait]
    impl Database for FailingDb {
        async fn get_version(&self) -> Result<String, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn latest_car(&self) -> Result<Option<Car>, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn list_cars(
            &self,
            _status: Option<&str>,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<Car>, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn sold_cars(&self, _limit: i64) -> Result<Vec<Car>, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn filter_cars(
            &self,
            _filter: &crate::core::models::CarFilter,
        ) -> Result<Vec<Car>, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn avg_deal_duration_days(&self) -> Result<Option<i64>, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn cars_stats(&self) -> Result<Stats, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn contacts(&self) -> Result<Option<Contact>, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn record_site_visit(&self) -> Result<(), sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn site_visits(&self) -> Result<crate::core::models::VisitStats, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn admin_list_cars(
            &self,
            _status: Option<&str>,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<Car>, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn car_by_id(&self, _id: i64) -> Result<Option<Car>, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn next_lot_number(&self) -> Result<String, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn create_car(&self, _new: &crate::core::models::NewCar) -> Result<Car, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn set_car_status(
            &self,
            _id: i64,
            _update: &crate::core::models::StatusUpdate,
        ) -> Result<Option<Car>, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn delete_car(&self, _id: i64) -> Result<Option<Vec<String>>, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn add_car_image(&self, _car_id: i64, _url: &str) -> Result<i64, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }

        async fn delete_car_image(&self, _car_id: i64, _url: &str) -> Result<bool, sqlx::Error> {
            Err(sqlx::Error::PoolTimedOut)
        }
    }

    /// Читает тело ответа как JSON — чтобы проверять данные, а не только код.
    async fn json_body(response: axum::response::Response) -> serde_json::Value {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    /// Читает тело ответа как текст — для HTML-фрагментов и статики.
    async fn text_body(response: axum::response::Response) -> String {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(body.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn test_db_handler_returns_500_and_hides_details() {
        let app = create_router(AppState::new(FailingDb, Config::for_tests()));

        let response = app
            .oneshot(Request::builder().uri("/db").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // Внутренности ошибки (пути, SQL) не должны утекать клиенту.
        let json = json_body(response).await;
        assert_eq!(json["message"], "Внутренняя ошибка сервера");
    }

    #[tokio::test]
    async fn test_api_cars_returns_only_active() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/api/cars")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = json_body(response).await;
        let cars = json.as_array().expect("ожидался массив автомобилей");
        // В реальных данных два активных объявления: Honda XR-V и Audi A3.
        assert_eq!(cars.len(), 2);
        assert!(cars.iter().all(|car| car["status"] != "sold"));
    }

    #[tokio::test]
    async fn test_api_cars_filtered_by_status() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/api/cars?status=stock")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = json_body(response).await;
        let cars = json.as_array().unwrap();
        // Оба реальных авто сейчас в наличии.
        assert_eq!(cars.len(), 2);
        assert!(cars.iter().all(|car| car["status"] == "stock"));
    }

    #[tokio::test]
    async fn test_api_cars_rejects_unknown_status() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/api/cars?status=hacked")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_api_cars_latest_returns_newest() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/api/cars/latest")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = json_body(response).await;
        // Самая свежая запись — реальное объявление Audi A3 (лот 0249).
        assert_eq!(json["lot_number"], "0249");
        // Обложка — относительный путь, который раздаёт сам бэкенд.
        let cover = json["image_url"]
            .as_str()
            .expect("image_url должен быть строкой");
        assert!(
            cover.starts_with("/images/"),
            "неожиданная обложка: {cover}"
        );
        // Галерея реального авто: 9 фотографий.
        assert_eq!(json["images"].as_array().map(Vec::len), Some(9));
    }

    #[tokio::test]
    async fn test_static_images_are_served() {
        // Раздачу статики проверяем на реальном каталоге `static/images`: тестовые
        // конфиги указывают на временный каталог, чтобы загрузка фото не трогала
        // репозиторий.
        let mut config = Config::for_tests();
        config.images_dir = "static/images".to_string();
        let db = SqliteDb::new("sqlite::memory:")
            .await
            .expect("Не удалось создать in-memory БД");

        let response = create_router(AppState::new(db, config))
            .oneshot(
                Request::builder()
                    .uri("/images/honda-xr-v-2023/honda-xr-v-2023-1.jpeg")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        assert!(
            content_type.starts_with("image/jpeg"),
            "ожидался image/jpeg, получено '{content_type}'"
        );
    }

    #[tokio::test]
    async fn test_api_cars_sold() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/api/cars/sold")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = json_body(response).await;
        let cars = json.as_array().unwrap();
        // Проданных в реальных данных пока нет.
        assert!(cars.is_empty());
    }

    #[tokio::test]
    async fn test_api_stats_counts_by_status() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/api/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = json_body(response).await;
        assert_eq!(json["total"], 2);
        assert_eq!(json["in_stock"], 2);
        assert_eq!(json["in_transit"], 0);
        assert_eq!(json["on_order"], 0);
        assert_eq!(json["sold"], 0);
    }

    #[tokio::test]
    async fn test_api_contacts() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/api/contacts")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = json_body(response).await;
        assert_eq!(json["phone_primary"], "89996832070");
    }

    /// GET /partials/hero — фрагмент главного акцента с последним авто и галереей.
    #[tokio::test]
    async fn test_partial_hero_renders_latest_car() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/partials/hero")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let html = text_body(response).await;
        assert!(html.contains("Audi A3"), "нет названия авто");
        assert!(html.contains("audi-a3-1.jpg"), "нет фото обложки");
        assert!(html.contains("hero-thumb"), "нет миниатюр галереи");
    }

    /// GET /partials/catalog — карточки по фильтру + счётчики через hx-swap-oob.
    #[tokio::test]
    async fn test_partial_catalog_filters_and_counts() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/partials/catalog?status=stock")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let html = text_body(response).await;
        // Оба реальных авто в наличии.
        assert_eq!(html.matches("class=\"cc rv\"").count(), 2);
        // Счётчики фильтров уезжают отдельными элементами (вне области подстановки).
        assert!(html.contains("id=\"countAll\""));
        assert!(html.contains("hx-swap-oob"));
        // Проданных в каталоге быть не должно.
        assert!(!html.contains("ПРОДАНО"));
    }

    /// Поиск в каталоге фильтрует по марке, модели и номеру лота.
    #[tokio::test]
    async fn test_partial_catalog_search() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/partials/catalog?q=xr-v")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let html = text_body(response).await;
        assert!(html.contains("Honda XR-V"));
        assert_eq!(html.matches("class=\"cc rv\"").count(), 1);
    }

    /// GET /partials/sold и /partials/stats отдают готовые блоки.
    #[tokio::test]
    async fn test_partial_sold_and_stats() {
        let app = test_app().await;

        let sold = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/partials/sold")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(sold.status(), StatusCode::OK);
        assert!(text_body(sold)
            .await
            .contains("Проданных автомобилей пока нет"));

        let stats = app
            .oneshot(
                Request::builder()
                    .uri("/partials/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(stats.status(), StatusCode::OK);
        let html = text_body(stats).await;
        assert!(html.contains("data-to=\"2\""), "нет общего числа авто");
        assert!(html.contains("data-to=\"0\""), "нет числа проданных");
        // Средний срок сделки без продаж не показываем — это была бы выдумка.
        assert!(!html.contains("средний срок сделки"));
    }

    /// GET /partials/contacts отдаёт контакты и обновляет телефон в шапке.
    #[tokio::test]
    async fn test_partial_contacts_updates_header_phone() {
        let response = test_app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/partials/contacts")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let html = text_body(response).await;
        assert!(html.contains("89996832070"));
        assert!(html.contains("id=\"headerPhone\""));
        assert!(html.contains("hx-swap-oob"));
    }

    /// Страница нового фронтенда раздаётся сервером на /ui/.
    #[tokio::test]
    async fn test_frontend_page_and_assets_are_served() {
        let app = test_app().await;

        let page = app
            .clone()
            .oneshot(Request::builder().uri("/ui/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(page.status(), StatusCode::OK);
        let html = text_body(page).await;
        assert!(html.contains("ВОСТОК АВТО"));
        assert!(html.contains("vendor/htmx.min.js"));

        let css = app
            .oneshot(
                Request::builder()
                    .uri("/ui/css/base.css")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(css.status(), StatusCode::OK);
        let content_type = css
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        assert!(
            content_type.starts_with("text/css"),
            "ожидался text/css, получено '{content_type}'"
        );
    }

    #[tokio::test]
    async fn test_api_stats_hides_db_error() {
        let app = create_router(AppState::new(FailingDb, Config::for_tests()));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let json = json_body(response).await;
        assert_eq!(json["message"], "Внутренняя ошибка сервера");
    }

    // ── Админ-API: управление каталогом ────────────────────────────────────

    /// Запрос к админ-API с корректным токеном (без тела).
    fn admin_request(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header("x-admin-token", crate::core::config::TEST_ADMIN_TOKEN)
            .header("content-type", "application/json")
            .body(Body::empty())
            .unwrap()
    }

    /// Запрос к админ-API с корректным токеном и JSON-телом.
    fn admin_request_body(method: &str, uri: &str, body: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header("x-admin-token", crate::core::config::TEST_ADMIN_TOKEN)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    /// Без токена и с неверным токеном админ-API недоступен.
    #[tokio::test]
    async fn test_admin_requires_token() {
        let app = test_app().await;

        let without = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/admin/cars")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(without.status(), StatusCode::UNAUTHORIZED);

        let wrong = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/admin/cars")
                    .header("x-admin-token", "wrong-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
    }

    /// Если токен не задан в конфиге, админ-API отвечает 503 (а не открывается).
    #[tokio::test]
    async fn test_admin_disabled_without_token() {
        let mut config = Config::for_tests();
        config.admin_token = None;

        let db = SqliteDb::new("sqlite::memory:").await.unwrap();
        let app = create_router(AppState::new(db, config));

        let response = app
            .oneshot(admin_request("GET", "/api/admin/cars"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    /// Добавление, появление в списке и удаление автомобиля.
    #[tokio::test]
    async fn test_admin_create_list_and_delete_car() {
        let app = test_app().await;
        let body = r#"{
            "brand": "Lada", "model": "Vesta", "year": 2022, "mileage": 30000,
            "engine": "1.6 (106 л.с.)", "transmission": "MT", "drive": "FWD",
            "price_rub": 1200000
        }"#;

        // Создание: лот сервер назначает сам, статус по умолчанию — stock.
        let created = app
            .clone()
            .oneshot(admin_request_body("POST", "/api/admin/cars", body))
            .await
            .unwrap();
        assert_eq!(created.status(), StatusCode::CREATED);
        let created = json_body(created).await;
        let id = created["id"].as_i64().expect("id должен быть числом");
        assert_eq!(created["lot_number"], "0250", "следующий свободный лот");
        assert_eq!(created["status"], "stock");

        // Новое авто видно в админ-списке.
        let list = app
            .clone()
            .oneshot(admin_request("GET", "/api/admin/cars"))
            .await
            .unwrap();
        let list = json_body(list).await;
        assert!(list.as_array().unwrap().iter().any(|car| car["id"] == id));

        // Удаление и повторное удаление (404).
        let deleted = app
            .clone()
            .oneshot(admin_request("DELETE", &format!("/api/admin/cars/{id}")))
            .await
            .unwrap();
        assert_eq!(deleted.status(), StatusCode::OK);

        let again = app
            .oneshot(admin_request("DELETE", &format!("/api/admin/cars/{id}")))
            .await
            .unwrap();
        assert_eq!(again.status(), StatusCode::NOT_FOUND);
    }

    /// Перевод авто в «продано» проставляет дату сделки и срок.
    #[tokio::test]
    async fn test_admin_set_status_sold() {
        let app = test_app().await;
        let body = r#"{
            "brand": "Nissan", "model": "Skyline", "year": 1999, "mileage": 120000,
            "engine": "2.5 (280 л.с.)", "transmission": "AT", "drive": "RWD",
            "price_rub": 2500000
        }"#;

        let created = app
            .clone()
            .oneshot(admin_request_body("POST", "/api/admin/cars", body))
            .await
            .unwrap();
        let id = json_body(created).await["id"].as_i64().unwrap();

        let sold = app
            .clone()
            .oneshot(admin_request_body(
                "PATCH",
                &format!("/api/admin/cars/{id}/status"),
                r#"{"status": "sold", "delivery_city": "Москва"}"#,
            ))
            .await
            .unwrap();
        assert_eq!(sold.status(), StatusCode::OK);
        let sold = json_body(sold).await;
        assert_eq!(sold["status"], "sold");
        assert_eq!(sold["delivery_city"], "Москва");
        assert!(sold["sold_at"].is_string(), "должна появиться дата продажи");
        assert!(sold["deal_duration_days"].is_number());

        // Проданных стало на одного больше (в реальных данных их пока не было).
        let stats = app
            .oneshot(admin_request("GET", "/api/stats"))
            .await
            .unwrap();
        assert_eq!(json_body(stats).await["sold"], 1);
    }

    /// Загрузка фото сохраняет файл и добавляет запись в галерею авто.
    #[tokio::test]
    async fn test_admin_upload_image() {
        let app = test_app().await;

        // Берём реальное авто из сида (последнее — Audi A3 с 9 фото).
        let latest = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/cars/latest")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let car = json_body(latest).await;
        let id = car["id"].as_i64().unwrap();
        let before = car["images"].as_array().unwrap().len();

        // Сервер не разбирает картинку, а проверяет MIME и размер, поэтому
        // тела достаточно, чтобы проверить путь загрузки.
        let payload: &[u8] = b"fake-jpeg-bytes";
        let boundary = "bec-boundary";
        let mut body = Vec::new();
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            b"Content-Disposition: form-data; name=\"file\"; filename=\"photo.jpg\"\r\n",
        );
        body.extend_from_slice(b"Content-Type: image/jpeg\r\n\r\n");
        body.extend_from_slice(payload);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        let request = Request::builder()
            .method("POST")
            .uri(format!("/api/admin/cars/{id}/images"))
            .header("x-admin-token", crate::core::config::TEST_ADMIN_TOKEN)
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let updated = json_body(response).await;
        let images = updated["images"].as_array().unwrap();
        assert_eq!(images.len(), before + 1, "в галерее появилось новое фото");
        let added = images.last().unwrap().as_str().unwrap();
        assert!(
            added.starts_with("/images/"),
            "неожиданный путь фото: {added}"
        );
    }

    // ── Счётчик посещений сайта ────────────────────────────────────────────

    /// Счётчик растёт от просмотров страницы `/ui/`, но не от статики.
    #[tokio::test]
    async fn test_site_visit_counter_counts_only_page_views() {
        let app = test_app().await;

        // Загружаем страницу и её CSS: считаться должен только HTML.
        let page = app
            .clone()
            .oneshot(Request::builder().uri("/ui/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(page.status(), StatusCode::OK);

        let css = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/ui/css/base.css")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(css.status(), StatusCode::OK);

        // Повторный просмотр страницы — второй в счётчике.
        let page = app
            .clone()
            .oneshot(Request::builder().uri("/ui/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(page.status(), StatusCode::OK);

        let visits = app
            .oneshot(admin_request("GET", "/api/admin/stats/visits"))
            .await
            .unwrap();
        assert_eq!(visits.status(), StatusCode::OK);
        let visits = json_body(visits).await;
        assert_eq!(visits["total"], 2, "статика не должна попадать в счётчик");
        assert_eq!(visits["today"], 2);
        assert_eq!(visits["last_7_days"], 2);
    }

    /// Счётчик посещений закрыт админ-токеном — публично его не видно.
    #[tokio::test]
    async fn test_visits_requires_admin_token() {
        let app = test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/admin/stats/visits")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Сбой БД в счётчике не ломает показ сайта.
    #[tokio::test]
    async fn test_site_visit_counter_survives_db_error() {
        let app = create_router(AppState::new(FailingDb, Config::for_tests()));

        let response = app
            .oneshot(Request::builder().uri("/ui/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
