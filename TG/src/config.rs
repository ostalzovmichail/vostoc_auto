//! Конфигурация бота: токен и каталог для сохранения файлов.

use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};

/// Каталог по умолчанию, если `DATA_DIR` не задан.
const DEFAULT_DATA_DIR: &str = "./data";

/// Настройки бота, прочитанные из `.env` и переменных окружения.
#[derive(Debug, Clone)]
pub struct Config {
    /// Токен бота от @BotFather.
    pub bot_token: String,
    /// Корневой каталог, куда складываются тексты и фотографии.
    pub data_dir: PathBuf,
}

impl Config {
    /// Загружает конфигурацию: подключает `.env`, затем читает переменные.
    pub fn load() -> Result<Self> {
        // Отсутствие .env — штатная ситуация (переменные могут прийти из
        // окружения). Любую другую ошибку разбора печатаем: молча
        // проигнорированный .env приводит к загадочному «токен не задан».
        match dotenvy::dotenv() {
            Ok(_) => {}
            Err(dotenvy::Error::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => eprintln!("[env] .env не загружен: {error}"),
        }

        // Токен: поддерживаем и «родное» имя teloxide, и короткое BOT_TOKEN.
        let bot_token = env::var("TELOXIDE_TOKEN")
            .or_else(|_| env::var("BOT_TOKEN"))
            .context("Не задан токен бота: укажите TELOXIDE_TOKEN в .env (см. .env.example)")?;

        let data_dir = env::var("DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_DATA_DIR));

        Ok(Self {
            bot_token,
            data_dir,
        })
    }
}
