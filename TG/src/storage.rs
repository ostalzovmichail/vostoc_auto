//! Сохранение содержимого сообщений в файлы на диске.
//!
//! Раскладка каталогов:
//!
//! ```text
//! data/
//!   next_id                                            — последний выданный id
//!   text/<chat_id>/<метка>_<id>[_<номер>].txt          — тексты и подписи
//!   photos/<chat_id>/<метка>_<id>[_<номер>].jpg        — фотографии
//! ```
//!
//! Файлы разделены по чатам, имя собирается по правилам из [`crate::naming`].

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use teloxide::net::Download;
use teloxide::prelude::*;
use tokio::sync::Mutex;

/// Файл, в котором хранится последний выданный id.
const ID_FILE: &str = "next_id";

/// Вид сохраняемого файла — определяет подкаталог внутри корня данных.
enum Kind {
    Text,
    Photo,
}

impl Kind {
    /// Имя подкаталога для этого вида.
    fn dir(&self) -> &'static str {
        match self {
            Kind::Text => "text",
            Kind::Photo => "photos",
        }
    }
}

/// Хранилище файлов на диске.
pub struct Storage {
    root: PathBuf,
    /// Последний выданный id. За мьютексом, потому что обработчики работают
    /// параллельно и id должны быть уникальными.
    last_id: Mutex<u64>,
}

impl Storage {
    /// Создаёт каталоги и восстанавливает счётчик id с прошлого запуска.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        for kind in [Kind::Text, Kind::Photo] {
            std::fs::create_dir_all(root.join(kind.dir()))
                .with_context(|| format!("Не удалось создать каталог {}", root.display()))?;
        }

        // Счётчик продолжаем с прошлого значения, иначе после перезапуска
        // id начались бы заново и файлы перезаписывались.
        let last_id = std::fs::read_to_string(root.join(ID_FILE))
            .ok()
            .and_then(|value| value.trim().parse().ok())
            .unwrap_or(0);

        Ok(Self {
            root,
            last_id: Mutex::new(last_id),
        })
    }

    /// Выдаёт следующий id и сразу сохраняет его на диск.
    pub async fn next_id(&self) -> Result<u64> {
        let mut last_id = self.last_id.lock().await;
        *last_id += 1;
        let id = *last_id;
        tokio::fs::write(self.root.join(ID_FILE), id.to_string())
            .await
            .with_context(|| "Не удалось сохранить счётчик id")?;
        Ok(id)
    }

    /// Сохраняет текст (сообщение или подпись) в файл `.txt`.
    pub async fn save_text(&self, chat_id: ChatId, file_name: &str, text: &str) -> Result<PathBuf> {
        let path = self.chat_dir(Kind::Text, chat_id).join(file_name);
        create_parent(&path).await?;
        tokio::fs::write(&path, text)
            .await
            .with_context(|| format!("Не удалось записать {}", path.display()))?;
        Ok(path)
    }

    /// Скачивает фотографию из Telegram и сохраняет её в `.jpg`.
    ///
    /// `remote_path` — путь, полученный из `Bot::get_file`.
    pub async fn save_photo(
        &self,
        bot: &Bot,
        remote_path: &str,
        chat_id: ChatId,
        file_name: &str,
    ) -> Result<PathBuf> {
        let path = self.chat_dir(Kind::Photo, chat_id).join(file_name);
        create_parent(&path).await?;
        let mut file = tokio::fs::File::create(&path)
            .await
            .with_context(|| format!("Не удалось создать {}", path.display()))?;
        bot.download_file(remote_path, &mut file)
            .await
            .with_context(|| format!("Не удалось скачать фото в {}", path.display()))?;
        Ok(path)
    }

    /// Путь относительно корня данных — его безопасно показывать в ответе.
    pub fn relative(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .display()
            .to_string()
    }

    /// Каталог файлов указанного вида для конкретного чата.
    fn chat_dir(&self, kind: Kind, chat_id: ChatId) -> PathBuf {
        self.root.join(kind.dir()).join(chat_id.to_string())
    }
}

/// Создаёт родительский каталог файла (папку конкретного чата).
async fn create_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Не удалось создать каталог {}", parent.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Уникальный временный каталог: имя привязано к процессу и названию теста.
    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tg_archiver_{}_{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn new_creates_subdirectories() {
        let root = temp_root("dirs");
        let _storage = Storage::new(&root).unwrap();

        assert!(root.join("text").is_dir(), "нет каталога text");
        assert!(root.join("photos").is_dir(), "нет каталога photos");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn save_text_writes_named_file() {
        let root = temp_root("text");
        let storage = Storage::new(&root).unwrap();

        let path = storage
            .save_text(ChatId(42), "Привет_7.txt", "привет")
            .await
            .unwrap();

        assert!(path.exists(), "файл не создан: {}", path.display());
        assert_eq!(
            storage.relative(&path).replace('\\', "/"),
            "text/42/Привет_7.txt"
        );
        assert_eq!(tokio::fs::read_to_string(&path).await.unwrap(), "привет");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn next_id_increments_and_survives_restart() {
        let root = temp_root("ids");
        let storage = Storage::new(&root).unwrap();

        assert_eq!(storage.next_id().await.unwrap(), 1);
        assert_eq!(storage.next_id().await.unwrap(), 2);

        // Новый Storage в том же каталоге продолжает нумерацию, а не начинает с 1.
        let restarted = Storage::new(&root).unwrap();
        assert_eq!(restarted.next_id().await.unwrap(), 3);

        let _ = std::fs::remove_dir_all(&root);
    }
}
