//! Работа с фотографиями автомобилей: загрузка, удаление, безопасные пути.
//!
//! Логика общая для JSON API (`/api/admin/cars/:id/images`) и HTML-админки:
//! два места, где сохраняются фото, не должны разойтись в проверках MIME,
//! размера и защите от выхода за пределы каталога.

use std::path::{Component, Path, PathBuf};

use axum::extract::Multipart;

use crate::api::AppState;
use crate::core::models::Car;

/// Максимальный размер загружаемого фото.
pub const MAX_IMAGE_BYTES: usize = 15 * 1024 * 1024;

/// Префикс, под которым раздаются фотографии (совпадает с `static_routes`).
pub const IMAGES_PREFIX: &str = "/images/";

/// Сохранённый на диске файл: путь для удаления при сбое записи в БД.
pub struct SavedImage {
    pub url: String,
    pub path: PathBuf,
}

/// Читает все файлы из multipart-запроса.
///
/// Возвращает пары `(расширение, байты)`. Ошибка — человекочитаемое сообщение,
/// которое можно показать в форме.
pub async fn read_images(mut multipart: Multipart) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut images = Vec::new();

    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(error) => {
                tracing::warn!(error = %error, "Некорректный multipart-запрос");
                return Err("Некорректный multipart-запрос".to_string());
            }
        };

        // Поле без имени файла — не файл: пропускаем.
        let Some(file_name) = field.file_name().map(str::to_string) else {
            continue;
        };
        let content_type = field.content_type().map(str::to_string);

        let Some(extension) = image_extension(content_type.as_deref(), &file_name) else {
            return Err("Поддерживаются только JPEG, PNG и WebP".to_string());
        };

        let bytes = match field.bytes().await {
            Ok(bytes) => bytes,
            Err(error) => {
                tracing::warn!(error = %error, "Не удалось прочитать загружаемый файл");
                return Err("Не удалось прочитать файл".to_string());
            }
        };

        if bytes.is_empty() {
            return Err("Пустой файл".to_string());
        }
        if bytes.len() > MAX_IMAGE_BYTES {
            return Err("Файл больше 15 МБ".to_string());
        }

        images.push((extension, bytes.to_vec()));
    }

    if images.is_empty() {
        return Err("В запросе нет файла".to_string());
    }

    Ok(images)
}

/// Сохраняет файл в каталог фото и возвращает URL и путь.
///
/// Каталог группируется по марке/модели/лоту. `slugify` оставляет только
/// ASCII-буквы, цифры и дефисы, поэтому выйти за пределы каталога нельзя.
pub async fn save_image(
    state: &AppState,
    car: &Car,
    extension: &str,
    bytes: &[u8],
) -> Result<SavedImage, String> {
    let slug = slugify(&format!("{}-{}-{}", car.brand, car.model, car.lot_number));
    let directory = Path::new(&state.config.images_dir).join(&slug);
    if let Err(error) = tokio::fs::create_dir_all(&directory).await {
        tracing::error!(error = %error, "Не удалось создать каталог для фото");
        return Err("Не удалось сохранить фото".to_string());
    }

    let file_name = format!("{slug}-{}.{extension}", image_stamp());
    let path = directory.join(&file_name);
    if let Err(error) = tokio::fs::write(&path, bytes).await {
        tracing::error!(error = %error, path = %path.display(), "Не удалось записать фото");
        return Err("Не удалось сохранить фото".to_string());
    }

    Ok(SavedImage {
        url: format!("/images/{slug}/{file_name}"),
        path,
    })
}

/// Удаляет файл сохранённого фото (best-effort).
pub async fn remove_saved(image: &SavedImage) {
    if let Err(error) = tokio::fs::remove_file(&image.path).await {
        if error.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!(error = %error, path = %image.path.display(), "Не удалось удалить файл фото");
        }
    }
}

/// Удаляет файл фото по URL из БД.
pub async fn remove_image_file(state: &AppState, url: &str) {
    let Some(relative) = url.strip_prefix(IMAGES_PREFIX) else {
        return;
    };
    match safe_join(&state.config.images_dir, relative) {
        Some(path) => {
            if let Err(error) = tokio::fs::remove_file(&path).await {
                if error.kind() != std::io::ErrorKind::NotFound {
                    tracing::warn!(error = %error, path = %path.display(), "Не удалось удалить файл фото");
                }
            }
        }
        None => tracing::warn!(url = %url, "Подозрительный путь фото — файл не удалён"),
    }
}

/// Удаляет каталог фото авто, если он опустел.
pub async fn remove_empty_dir(state: &AppState, urls: &[String]) {
    // Все фото одного авто лежат в одном каталоге — берём его из первого пути.
    let Some(relative) = urls.first().and_then(|url| url.strip_prefix(IMAGES_PREFIX)) else {
        return;
    };
    let Some(parent) = Path::new(relative).parent() else {
        return;
    };
    if parent.as_os_str().is_empty() {
        return;
    }
    let Some(path) = safe_join(&state.config.images_dir, &parent.to_string_lossy()) else {
        return;
    };

    // remove_dir сработает только если каталог пуст: иначе оставляем как есть.
    if let Err(error) = tokio::fs::remove_dir(&path).await {
        if error.kind() != std::io::ErrorKind::NotFound {
            tracing::debug!(error = %error, path = %path.display(), "Каталог фото не удалён");
        }
    }
}

/// Определяет расширение файла по MIME-типу, с запасным вариантом — по имени.
pub fn image_extension(content_type: Option<&str>, file_name: &str) -> Option<String> {
    if let Some(content_type) = content_type {
        let essence = content_type
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        match essence.as_str() {
            "image/jpeg" | "image/jpg" => return Some("jpg".to_string()),
            "image/png" => return Some("png".to_string()),
            "image/webp" => return Some("webp".to_string()),
            _ => {}
        }
    }

    let extension = Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())?
        .to_ascii_lowercase();
    match extension.as_str() {
        "jpg" | "jpeg" => Some("jpg".to_string()),
        "png" => Some("png".to_string()),
        "webp" => Some("webp".to_string()),
        _ => None,
    }
}

/// Приводит строку к безопасному имени каталога: ASCII-буквы, цифры, дефисы.
pub fn slugify(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut previous_dash = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            previous_dash = false;
        } else if !out.is_empty() && !previous_dash {
            out.push('-');
            previous_dash = true;
        }
    }

    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "car".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Имя файла делаем уникальным: время в наносекундах + счётчик.
///
/// Одного времени в миллисекундах мало: несколько фото из одного запроса
/// сохраняются в цикле и легко попадают в одну миллисекунду, перетирая друг
/// друга. Счётчик гарантирует разные имена даже при одинаковом времени.
fn image_stamp() -> u128 {
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed) as u128;
    now * 1000 + sequence
}

/// Присоединяет относительный путь к базовому, отвергая `..` и абсолютные пути.
pub fn safe_join(base: &str, relative: &str) -> Option<PathBuf> {
    let mut path = PathBuf::from(base);
    for component in Path::new(relative).components() {
        match component {
            Component::Normal(part) => path.push(part),
            Component::CurDir => {}
            // Корень, `..` и префиксы (Windows) запрещены.
            _ => return None,
        }
    }
    Some(path)
}
