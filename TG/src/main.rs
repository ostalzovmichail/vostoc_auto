//! Телеграм-бот: сохраняет содержимое сообщений и фотографии на диск.
//!
//! Что делает бот:
//! * текстовое сообщение → файл `.txt` с его содержимым;
//! * фотографию → файл `.jpg` (наибольший доступный размер);
//! * подпись к фото → отдельный `.txt` с тем же именем;
//! * фото, отправленное «как файл» (JPEG) → тоже `.jpg`;
//! * **альбом** (несколько фото одним сообщением) → все файлы получают общий id
//!   и различаются номером.
//!
//! Имя файла — `<метка>_<id>[_<номер>].<расширение>`: метка — это первое слово
//! первой строки текста или подписи, id — сквозной номер, который бот
//! присваивает при сохранении (см. `naming`).

mod album;
mod config;
mod naming;
mod storage;

use std::sync::Arc;

use anyhow::Result;
use teloxide::prelude::*;
use teloxide::types::{Document, FileId, PhotoSize};
use tracing_subscriber::{fmt, EnvFilter};

use crate::album::AlbumBuffer;
use crate::config::Config;
use crate::storage::Storage;

/// Подсказка на команды `/start` и `/help`.
const HELP: &str = "Отправьте мне текст или фотографию — я сохраню их на диск.\n\
    • текст → файл .txt\n\
    • фото → файл .jpg\n\
    • альбом из нескольких фото → общий id, файлы различаются номером\n\
    Имя файла: <первое слово>_<id>. Например: Привет_7.txt";

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    // Конфигурация: токен бота и каталог для сохранения.
    let config = Config::load()?;
    let storage = Arc::new(Storage::new(&config.data_dir)?);
    let albums = AlbumBuffer::new();
    let bot = Bot::new(&config.bot_token);

    // Проверяем токен сразу: неверный токен — понятная ошибка на старте,
    // а не «тишина» после запуска.
    let me = bot.get_me().await?;
    tracing::info!(
        username = %me.username(),
        data_dir = %config.data_dir.display(),
        "Бот запущен"
    );

    // Единая точка входа: тип сообщения разбираем внутри обработчика. Так все
    // ветки видны в одном месте и не зависим от набора фильтров dptree.
    let handler = Update::filter_message().endpoint(handle_message);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![storage, albums])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

/// Разбирает сообщение по типу содержимого.
async fn handle_message(
    bot: Bot,
    msg: Message,
    storage: Arc<Storage>,
    albums: Arc<AlbumBuffer>,
) -> Result<()> {
    if let Some(text) = msg.text() {
        return handle_text(&bot, &msg, text, &storage).await;
    }
    if let Some(photos) = msg.photo() {
        return handle_photo(&bot, &msg, photos, &storage, &albums).await;
    }
    if let Some(document) = msg.document() {
        return handle_document(&bot, &msg, document, &storage, &albums).await;
    }
    // Стикеры, голосовые, видео и прочее не сохраняем.
    Ok(())
}

/// Текст: команда → подсказка, обычный текст → файл `.txt`.
async fn handle_text(bot: &Bot, msg: &Message, text: &str, storage: &Arc<Storage>) -> Result<()> {
    if text.starts_with('/') {
        bot.send_message(msg.chat.id, HELP).await?;
        return Ok(());
    }

    let tag = naming::tag_from(text).unwrap_or_else(|| naming::FALLBACK_TEXT.to_string());
    let id = storage.next_id().await?;
    let name = naming::file_name(&tag, id, None, "txt");
    let path = storage.save_text(msg.chat.id, &name, text).await?;

    tracing::info!(path = %path.display(), "Сохранён текст");
    bot.send_message(
        msg.chat.id,
        format!("✅ Текст сохранён:\n{}", storage.relative(&path)),
    )
    .await?;
    Ok(())
}

/// Фото: берём наибольший размер и сохраняем.
async fn handle_photo(
    bot: &Bot,
    msg: &Message,
    photos: &[PhotoSize],
    storage: &Arc<Storage>,
    albums: &Arc<AlbumBuffer>,
) -> Result<()> {
    // Telegram присылает несколько размеров; берём самый крупный по площади.
    let Some(best) = photos
        .iter()
        .max_by_key(|photo| u64::from(photo.width) * u64::from(photo.height))
    else {
        return Ok(());
    };

    save_image(bot, msg, best.file.id.clone(), storage, albums).await
}

/// Документ: сохраняем только JPEG (фото, отправленные «как файл»).
async fn handle_document(
    bot: &Bot,
    msg: &Message,
    document: &Document,
    storage: &Arc<Storage>,
    albums: &Arc<AlbumBuffer>,
) -> Result<()> {
    let is_jpeg = document
        .mime_type
        .as_ref()
        .is_some_and(|mime| mime.essence_str() == "image/jpeg");

    if !is_jpeg {
        bot.send_message(msg.chat.id, "Сохраняю только текст и фото (JPEG).")
            .await?;
        return Ok(());
    }

    save_image(bot, msg, document.file.id.clone(), storage, albums).await
}

/// Общий путь сохранения изображения: одиночное — сразу, из альбома — пачкой.
async fn save_image(
    bot: &Bot,
    msg: &Message,
    file_id: FileId,
    storage: &Arc<Storage>,
    albums: &Arc<AlbumBuffer>,
) -> Result<()> {
    let caption = msg.caption();

    // Альбом (несколько фото одним сообщением): копим и сохраняем пачкой —
    // так все файлы получают общий id и метку из подписи.
    if let Some(group_id) = msg.media_group_id() {
        let group_id = group_id.to_string();
        let tick = albums
            .push(&group_id, storage, msg.chat.id, file_id, caption)
            .await?;
        albums.schedule_flush(
            group_id,
            tick,
            bot.clone(),
            Arc::clone(storage),
            msg.chat.id,
        );
        return Ok(());
    }

    // Одиночное изображение — сохраняем сразу.
    let tag = caption
        .and_then(naming::tag_from)
        .unwrap_or_else(|| naming::FALLBACK_PHOTO.to_string());
    let id = storage.next_id().await?;

    let file = bot.get_file(file_id).await?;
    let name = naming::file_name(&tag, id, None, "jpg");
    let path = storage
        .save_photo(bot, &file.path, msg.chat.id, &name)
        .await?;

    // Подпись — отдельным .txt с тем же именем.
    if let Some(caption) = caption {
        let text_name = naming::file_name(&tag, id, None, "txt");
        storage.save_text(msg.chat.id, &text_name, caption).await?;
    }

    tracing::info!(path = %path.display(), "Сохранено фото");
    bot.send_message(
        msg.chat.id,
        format!("✅ Фото сохранено:\n{}", storage.relative(&path)),
    )
    .await?;
    Ok(())
}

/// Логи в stdout; уровень задаётся `RUST_LOG` (по умолчанию `info`).
fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).with_target(false).init();
}
