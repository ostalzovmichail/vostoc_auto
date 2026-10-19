use std::env;

/// Каталог со статикой (фотографии авто) по умолчанию.
///
/// Используется и при раздаче `/images`, и при загрузке фото из админ-API —
/// чтобы каталог чтения и записи всегда совпадал.
pub const DEFAULT_IMAGES_DIR: &str = "static/images";

/// Базовый адрес API OpenRouter (OpenAI-совместимый `/chat/completions`).
///
/// Вынесен в константу, чтобы в тестах подменять адрес мок-сервером, а не
/// уходить в реальный провайдер.
pub const DEFAULT_OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";

/// Модель OpenRouter по умолчанию: быстрая и дешёвая — для коротких
/// маркетинговых текстов её возможностей достаточно.
pub const DEFAULT_OPENROUTER_MODEL: &str = "openai/gpt-4o-mini";

/// Значение токена администратора в тестах.
#[cfg(test)]
pub const TEST_ADMIN_TOKEN: &str = "test-admin-token";

/// Поддерживаемые окружения приложения.
///
/// Окружение выбирается переменной `APP_ENV` ("dev" по умолчанию) и влияет
/// на логику, которую нельзя смешивать в разных местах кода: фильтр логирования,
/// доступность Swagger UI, поведение CORS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    /// Разбирает строку из `APP_ENV`, допуская и короткие, и полные имена.
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "dev" | "development" => Ok(Self::Development),
            "prod" | "production" => Ok(Self::Production),
            other => Err(format!(
                "Неизвестное значение APP_ENV='{other}'. Допустимые значения: dev, prod"
            )),
        }
    }

    /// Короткое имя окружения — используется и для имени файла `.env.{name}`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Development => "dev",
            Self::Production => "prod",
        }
    }

    pub fn is_dev(self) -> bool {
        matches!(self, Self::Development)
    }
}

/// Конфигурация приложения, загружаемая из переменных окружения.
///
/// Валидация выполняется один раз при старте: если конфигурация невалидна,
/// процесс завершается сразу (`Config::load` возвращает `Err` с понятным
/// сообщением), а не падает где-то посреди работы.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub start_message: String,
    /// Разрешённые CORS-origin'ы. Используются только в prod;
    /// в dev включается permissive-режим (см. `api::router::cors_layer`).
    pub cors_allowed_origins: Vec<String>,
    /// Токен администратора для админ-API (заголовок `X-Admin-Token`).
    ///
    /// `None` — админ-API полностью отключён (маршруты отвечают 503), но сам
    /// сайт продолжает работать. Так неготовность настроить токен не роняет
    /// публичную часть, а админку нельзя случайно оставить открытой.
    pub admin_token: Option<String>,
    /// Каталог, где лежат фотографии авто и куда пишет загрузка из админ-API.
    pub images_dir: String,
    /// Ключ OpenRouter для AI-функций (`/api/admin/ai/*`).
    ///
    /// `None` — AI-эндпоинты отвечают 503, остальная работа сервера не
    /// затрагивается. Ключ живёт только на сервере и никогда не отдаётся
    /// клиенту: админ-приложение знает лишь адрес нашего API.
    pub openrouter_api_key: Option<String>,
    /// Модель OpenRouter по умолчанию (переопределяется `OPENROUTER_MODEL`).
    pub openrouter_model: String,
    /// Базовый адрес OpenRouter; нужен для подмены в тестах (мок-сервер).
    pub openrouter_base_url: String,
    pub environment: Environment,
}

impl Config {
    /// Загружает конфигурацию из окружения и файлов `.env.*`.
    pub fn load() -> Result<Self, String> {
        // Шаг 1: определяем окружение. Оно нужно ещё до загрузки .env-файлов,
        // потому что имя файла зависит от него.
        let environment = match env::var("APP_ENV") {
            Ok(raw) => Environment::parse(&raw)?,
            // APP_ENV не задан — используем значение по умолчанию.
            Err(_) => Environment::Development,
        };

        // Шаг 2: подключаем файлы окружения.
        Self::load_env_files(environment);

        // Шаг 3: читаем и валидируем переменные.
        Ok(Self {
            database_url: require_var("DATABASE_URL")?,
            server_host: require_var("SERVER_HOST")?,
            server_port: parse_port(&require_var("SERVER_PORT")?)?,
            start_message: require_var("START_MESSAGE")?,
            cors_allowed_origins: parse_origin_list(
                &env::var("CORS_ALLOWED_ORIGINS").unwrap_or_default(),
            ),
            admin_token: optional_var("ADMIN_TOKEN"),
            images_dir: optional_var("IMAGES_DIR")
                .unwrap_or_else(|| DEFAULT_IMAGES_DIR.to_string()),
            openrouter_api_key: optional_var("OPENROUTER_API_KEY"),
            openrouter_model: optional_var("OPENROUTER_MODEL")
                .unwrap_or_else(|| DEFAULT_OPENROUTER_MODEL.to_string()),
            openrouter_base_url: optional_var("OPENROUTER_BASE_URL")
                .unwrap_or_else(|| DEFAULT_OPENROUTER_BASE_URL.to_string()),
            environment,
        })
    }

    /// Включена ли интерактивная документация (Swagger UI).
    /// В проде документация недоступна: она раскрывает поверхность API.
    pub fn docs_enabled(&self) -> bool {
        self.environment.is_dev()
    }

    /// Настроен ли админ-API (задан токен).
    pub fn admin_enabled(&self) -> bool {
        self.admin_token.is_some()
    }

    /// Компоненты адреса для привязки TCP-слушателя.
    pub fn bind_addr(&self) -> (&str, u16) {
        (&self.server_host, self.server_port)
    }

    /// Загружает `.env.{APP_ENV}`, затем `.env` как резервный файл.
    ///
    /// dotenvy не перезаписывает уже заданные переменные, поэтому приоритет
    /// такой: системное окружение > `.env.{APP_ENV}` > `.env`.
    ///
    /// Отсутствие файла — штатная ситуация (значения могут прийти из окружения),
    /// поэтому «не найден» молчим. Любую другую ошибку разбора печатаем:
    /// молча проигнорированный `.env` — это неочевидно неверный конфиг, который
    /// раньше приводил к загадочному «переменная не задана» при старте.
    fn load_env_files(environment: Environment) {
        Self::load_env_file(&format!(".env.{}", environment.name()));
        Self::load_env_file(".env");
    }

    /// Подключает один env-файл, различая «файла нет» и ошибку разбора.
    fn load_env_file(filename: &str) {
        match dotenvy::from_filename(filename) {
            Ok(path) => eprintln!("[env] загружен {}", path.display()),
            Err(dotenvy::Error::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => eprintln!("[env] {filename}: {error}"),
        }
    }

    /// Конфигурация для юнит-тестов: изолированная in-memory БД, dev-режим,
    /// отдельный временный каталог для фото (чтобы тесты не трогали `static/`).
    #[cfg(test)]
    pub fn for_tests() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        // Уникальный каталог на каждый вызов: тесты идут параллельно, и общий
        // каталог приводил бы к гонкам при загрузке фото.
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let images_dir =
            env::temp_dir().join(format!("bec_test_images_{}_{}", std::process::id(), unique));

        Self {
            database_url: "sqlite::memory:".to_string(),
            server_host: "127.0.0.1".to_string(),
            server_port: 0,
            start_message: "test".to_string(),
            cors_allowed_origins: Vec::new(),
            admin_token: Some(TEST_ADMIN_TOKEN.to_string()),
            images_dir: images_dir.to_string_lossy().into_owned(),
            // AI по умолчанию выключен: тесты, которым он нужен, включают его
            // сами и подменяют адрес мок-сервером.
            openrouter_api_key: None,
            openrouter_model: DEFAULT_OPENROUTER_MODEL.to_string(),
            openrouter_base_url: DEFAULT_OPENROUTER_BASE_URL.to_string(),
            environment: Environment::Development,
        }
    }
}

/// Читает обязательную переменную окружения; её отсутствие — ошибка конфигурации.
fn require_var(name: &str) -> Result<String, String> {
    env::var(name).map_err(|_| {
        format!("Переменная {name} не задана: укажите её в окружении или в файле .env")
    })
}

/// Читает необязательную переменную; пустая строка трактуется как «не задана».
fn optional_var(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// `SERVER_PORT` хранится строкой, но нужен как число — валидируем сразу.
fn parse_port(raw: &str) -> Result<u16, String> {
    raw.parse().map_err(|_| {
        format!("SERVER_PORT='{raw}' не является портом (ожидается число от 0 до 65535)")
    })
}

/// Разбирает список origin'ов из `CORS_ALLOWED_ORIGINS="https://a.ru, https://b.ru"`.
/// Пустые элементы отбрасываются, чтобы двойные запятые не давали мусор.
fn parse_origin_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}
