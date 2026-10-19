//! AI-функции админ-API: генерация текстов через OpenRouter.
//!
//! Ключ OpenRouter живёт только на сервере (переменная `OPENROUTER_API_KEY`):
//! клиент обращается к `/api/admin/ai/*` под своим `X-Admin-Token` и никогда
//! не видит ключ провайдера. OpenRouter совместим с OpenAI API, поэтому
//! используется обычный `/chat/completions`.

use std::time::Duration;

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::api::models::{
    bad_gateway, bad_request, internal_error, service_unavailable, ApiResult,
};
use crate::api::AppState;

/// Таймаут запроса к OpenRouter.
///
/// Генерация текста — не мгновенная операция, но зависший провайдер не должен
/// подвешивать админку: за минуту модель либо ответила, либо мы отдаём
/// клиенту понятную ошибку и просим повторить.
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(60);

/// Ограничение на длину ответа модели.
///
/// Описание автомобиля — короткий маркетинговый текст, несколько сотен токенов
/// достаточно с большим запасом (и дешевле).
const MAX_TOKENS: u32 = 700;

/// Тело запроса: характеристики авто, из которых модель собирает объявление.
///
/// Те же поля, что и у `NewCar`, но без лота/статуса — на текст они не влияют.
#[derive(Debug, Deserialize)]
pub struct DescribeCarRequest {
    pub brand: String,
    pub model: String,
    pub year: i64,
    pub mileage: i64,
    pub engine: String,
    pub transmission: String,
    pub drive: String,
    pub price_rub: i64,
}

/// Ответ: готовый текст объявления и имя реально использованной модели.
#[derive(Debug, Serialize)]
pub struct DescribeCarResponse {
    pub text: String,
    pub model: String,
}

/// Системная инструкция модели.
///
/// Строгие рамки (без приветствий, без markdown, без выдуманных фактов)
/// нужны, чтобы ответ можно было скопировать в объявление «как есть».
const SYSTEM_PROMPT: &str = "\
Ты — копирайтер автосалона «ВОСТОК АВТО». Пиши на русском языке короткое \
маркетинговое описание автомобиля для каталога и площадок объявлений.

Требования:
- 4–6 предложений: сильные стороны авто, состояние, для кого подойдёт;
- используй ТОЛЬКО переданные характеристики, ничего не выдумывай;
- без приветствий, заголовков, эмодзи, markdown и нумерации;
- цену пиши в рублях, цифрами, в падеже по контексту;
- в конце предложи обратиться в салон для осмотра или записи на тест-драйв.";

/// POST /api/admin/ai/description — сгенерировать объявление по характеристикам.
///
/// Маршрут защищён админ-токеном (см. `admin_routes` в `api::router`):
/// генерация стоит денег, и доступ к ней должен быть ровно такой же,
/// как к управлению каталогом.
pub async fn describe_handler(
    State(state): State<AppState>,
    Json(request): Json<DescribeCarRequest>,
) -> Result<Json<DescribeCarResponse>, ApiResult> {
    // Без ключа на сервере AI не работает — это настройка, а не сбой,
    // поэтому 503, а не 500.
    let Some(api_key) = state.config.openrouter_api_key.as_deref() else {
        tracing::warn!("Запрос к AI при OPENROUTER_API_KEY не задан");
        return Err(service_unavailable(
            "AI-функции отключены: на сервере не задан OPENROUTER_API_KEY",
        ));
    };

    validate(&request)?;

    let model = state.config.openrouter_model.clone();
    let text = generate_description(&state, api_key, &request).await?;

    tracing::info!(brand = %request.brand, model = %model, "Описание автомобиля сгенерировано");
    Ok(Json(DescribeCarResponse { text, model }))
}

/// Проверка обязательных полей: пустые характеристики дают бессмысленный текст.
fn validate(request: &DescribeCarRequest) -> Result<(), ApiResult> {
    if request.brand.trim().is_empty() || request.model.trim().is_empty() {
        return Err(bad_request("Укажите марку и модель автомобиля"));
    }
    if request.year <= 0 || request.price_rub <= 0 {
        return Err(bad_request(
            "Год выпуска и цена должны быть положительными числами",
        ));
    }
    Ok(())
}

/// Обращается к OpenRouter и возвращает готовый текст описания.
async fn generate_description(
    state: &AppState,
    api_key: &str,
    request: &DescribeCarRequest,
) -> Result<String, ApiResult> {
    let client = reqwest::Client::builder()
        .timeout(UPSTREAM_TIMEOUT)
        .build()
        // Ошибка сборки HTTP-клиента — это сбой окружения, не запроса:
        // пишем в лог и отдаём общую 500.
        .map_err(|error| {
            tracing::error!(error = %error, "Не удалось создать HTTP-клиент для AI");
            internal_error()
        })?;

    let prompt = user_prompt(request);
    let upstream = ChatRequest {
        model: state.config.openrouter_model.as_str(),
        messages: vec![
            ChatMessage {
                role: "system",
                content: SYSTEM_PROMPT,
            },
            ChatMessage {
                role: "user",
                content: &prompt,
            },
        ],
        max_tokens: MAX_TOKENS,
        temperature: 0.7,
    };

    let url = format!("{}/chat/completions", state.config.openrouter_base_url);
    let response = client
        .post(&url)
        .bearer_auth(api_key)
        // OpenRouter просит указывать origin и название приложения.
        .header("HTTP-Referer", "https://bereg.local")
        .header("X-Title", "ВОСТОК АВТО · Управление")
        .json(&upstream)
        .send()
        .await
        .map_err(|error| map_send_error(&error))?;

    let status = response.status();
    if !status.is_success() {
        // Тело ошибки провайдера логируем, но клиенту не отдаём:
        // оно может содержать детали аккаунта (лимиты, баланс).
        let details = response.text().await.unwrap_or_default();
        tracing::error!(%status, details = %details, "OpenRouter вернул ошибку");
        return Err(bad_gateway("AI-сервис вернул ошибку, попробуйте позже"));
    }

    let body: serde_json::Value = response.json().await.map_err(|error| {
        tracing::error!(error = %error, "Некорректный JSON от OpenRouter");
        bad_gateway("AI-сервис вернул некорректный ответ")
    })?;

    parse_description(&body)
}

/// Собирает запрос пользователя из характеристик — по ним модель пишет текст.
fn user_prompt(request: &DescribeCarRequest) -> String {
    format!(
        "Сгенерируй описание для каталога автомобиля с такими характеристиками:\n\
         Марка: {}\n\
         Модель: {}\n\
         Год выпуска: {}\n\
         Пробег: {} км\n\
         Двигатель: {}\n\
         Коробка передач: {}\n\
         Привод: {}\n\
         Цена: {} ₽",
        request.brand.trim(),
        request.model.trim(),
        request.year,
        request.mileage,
        request.engine.trim(),
        request.transmission.trim(),
        request.drive.trim(),
        request.price_rub,
    )
}

/// Разбирает ответ `/chat/completions` в текст описания.
///
/// Берём `choices[0].message.content` — совместимо с форматом OpenAI и с
/// любыми дополнительными полями, которые провайдер добавит в ответ.
fn parse_description(body: &serde_json::Value) -> Result<String, ApiResult> {
    let text = body
        .pointer("/choices/0/message/content")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();

    if text.is_empty() {
        tracing::error!(body = %body, "Ответ OpenRouter без текста описания");
        return Err(bad_gateway(
            "AI-сервис вернул пустой ответ, попробуйте ещё раз",
        ));
    }

    Ok(text)
}

/// Трансформирует ошибку отправки запроса в человекочитаемый ответ.
fn map_send_error(error: &reqwest::Error) -> ApiResult {
    if error.is_timeout() {
        tracing::warn!(error = %error, "Таймаут запроса к OpenRouter");
        return bad_gateway("AI-сервис не ответил вовремя, попробуйте ещё раз");
    }
    tracing::error!(error = %error, "Не удалось обратиться к OpenRouter");
    bad_gateway("AI-сервис недоступен, попробуйте позже")
}

/// Сообщение модели: характеристики авто, собранные в один блок.
///
/// Отдельная структура, а не `HashMap`: формат запроса — это контракт
/// с провайдером, и компилятор не даст в нём опечататься.
#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    max_tokens: u32,
    temperature: f32,
}

/// Одно сообщение диалога (`system` — инструкция, `user` — запрос).
#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::{Config, TEST_ADMIN_TOKEN};
    use crate::core::db::SqliteDb;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
        routing::post,
        Router,
    };
    use tower::ServiceExt;

    /// Запрос к AI-эндпоинту с админ-токеном и JSON-телом.
    fn ai_request(body: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/api/admin/ai/description")
            .header("x-admin-token", TEST_ADMIN_TOKEN)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    /// Корректное тело запроса с характеристиками тестового авто.
    const VALID_BODY: &str = r#"{
        "brand": "Audi", "model": "A3", "year": 2021, "mileage": 55200,
        "engine": "1.4T (150 л.с.)", "transmission": "S tronic", "drive": "FWD",
        "price_rub": 1940000
    }"#;

    /// Собирает приложение поверх in-memory БД с заданной конфигурацией.
    async fn app_with(config: Config) -> Router {
        let db = SqliteDb::new("sqlite::memory:")
            .await
            .expect("Не удалось создать in-memory БД");
        crate::api::router::create_router(AppState::new(db, config))
    }

    /// Конфиг с включённым AI и адресом мок-сервера OpenRouter.
    fn ai_config(base_url: &str) -> Config {
        let mut config = Config::for_tests();
        config.openrouter_api_key = Some("test-openrouter-key".to_string());
        config.openrouter_model = "test/model".to_string();
        config.openrouter_base_url = base_url.to_string();
        config
    }

    /// Поднимает мок OpenRouter: отвечает на `/chat/completions` готовым
    /// ответом в формате провайдера и возвращает его адрес.
    async fn mock_openrouter() -> String {
        let app = Router::new().route(
            "/v1/chat/completions",
            post(|| async {
                axum::Json(serde_json::json!({
                    "id": "gen-1",
                    "model": "test/model",
                    "choices": [{
                        "index": 0,
                        "message": {
                            "role": "assistant",
                            "content": "Компактный Audi A3 2021 года… Обратитесь в салон."
                        }
                    }]
                }))
            }),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Не удалось поднять мок-сервер");
        let address = listener.local_addr().expect("нет адреса мок-сервера");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("мок-сервер упал");
        });

        format!("http://{address}/v1")
    }

    /// Читает тело ответа как JSON.
    async fn json_body(response: axum::response::Response) -> serde_json::Value {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    /// Без ключа AI-эндпоинт честно отвечает 503, а не падает.
    #[tokio::test]
    async fn test_describe_returns_503_without_api_key() {
        let app = app_with(Config::for_tests()).await;

        let response = app.oneshot(ai_request(VALID_BODY)).await.unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let json = json_body(response).await;
        assert!(json["message"]
            .as_str()
            .unwrap()
            .contains("OPENROUTER_API_KEY"));
    }

    /// Полный путь: запрос дошёл до OpenRouter и вернул текст описания.
    #[tokio::test]
    async fn test_describe_returns_generated_text() {
        let base_url = mock_openrouter().await;
        let app = app_with(ai_config(&base_url)).await;

        let response = app.oneshot(ai_request(VALID_BODY)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let json = json_body(response).await;
        assert!(json["text"].as_str().unwrap().contains("Audi A3"));
        assert_eq!(json["model"], "test/model");
    }

    /// Пустые марка/модель отклоняются до ухода в сеть.
    #[tokio::test]
    async fn test_describe_rejects_empty_brand() {
        let base_url = mock_openrouter().await;
        let app = app_with(ai_config(&base_url)).await;

        let body = VALID_BODY.replace("\"Audi\"", "\"  \"");
        let response = app.oneshot(ai_request(&body)).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// Битая модель OpenRouter не роняет сервер: клиент получает 502.
    #[tokio::test]
    async fn test_describe_maps_upstream_failure_to_502() {
        // Мок, который всегда отвечает 401 — например, неверный ключ.
        let app_mock = Router::new().route(
            "/v1/chat/completions",
            post(|| async {
                (
                    StatusCode::UNAUTHORIZED,
                    axum::Json(serde_json::json!({"error": {"message": "bad key"}})),
                )
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app_mock).await.unwrap();
        });

        let app = app_with(ai_config(&format!("http://{address}/v1"))).await;
        let response = app.oneshot(ai_request(VALID_BODY)).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    /// Таймаут/недоступность провайдера тоже превращается в понятный 502.
    #[tokio::test]
    async fn test_describe_maps_unreachable_upstream_to_502() {
        // Порт, на котором никто не слушает: соединение отклоняется сразу.
        let app = app_with(ai_config("http://127.0.0.1:9/api/v1")).await;

        let response = app.oneshot(ai_request(VALID_BODY)).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    /// Без админ-токена AI-эндпоинт закрыт, как и весь админ-API.
    #[tokio::test]
    async fn test_describe_requires_admin_token() {
        let base_url = mock_openrouter().await;
        let app = app_with(ai_config(&base_url)).await;

        let request = Request::builder()
            .method("POST")
            .uri("/api/admin/ai/description")
            .header("content-type", "application/json")
            .body(Body::from(VALID_BODY.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
