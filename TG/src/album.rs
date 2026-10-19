//! Сборка альбомов (media group).
//!
//! Telegram присылает альбом из нескольких фото как отдельные сообщения с общим
//! `media_group_id`, причём подпись обычно есть только у одного из них. Чтобы
//! все файлы альбома получили один id и одну метку, фото сначала накапливаются,
//! а сохраняются после короткой паузы — когда новых фото в группе уже не приходит.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use teloxide::prelude::*;
use teloxide::types::{ChatId, FileId};
use tokio::sync::Mutex;

use crate::naming;
use crate::storage::Storage;

/// Пауза после последнего фото, по истечении которой альбом сохраняется.
const DEBOUNCE: Duration = Duration::from_millis(1000);

/// Буфер незавершённых альбомов.
pub struct AlbumBuffer {
    inner: Mutex<HashMap<String, Pending>>,
}

/// Накопленные данные одного альбома.
struct Pending {
    /// Общий id альбома.
    id: u64,
    /// Чат, куда складывать файлы и отвечать.
    chat_id: ChatId,
    /// Метка из первой встреченной подписи.
    tag: Option<String>,
    /// Сама подпись — сохраняется отдельным `.txt` с тем же id.
    caption: Option<String>,
    /// Фото в порядке поступления.
    files: Vec<FileId>,
    /// Счётчик поступлений: по нему определяется «последнее» фото.
    tick: u64,
}

/// Итог сохранения альбома — для ответа в чат.
pub struct SavedAlbum {
    pub id: u64,
    pub tag: String,
    pub photos: usize,
}

impl AlbumBuffer {
    /// Создаёт буфер. Хранится в `Arc`, потому что таймеры должны жить дольше
    /// одного вызова обработчика.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(HashMap::new()),
        })
    }

    /// Регистрирует фото альбома и возвращает «тик» для последующего сброса.
    ///
    /// Первое фото группы получает для неё новый id; остальные переиспользуют его.
    pub async fn push(
        &self,
        group_id: &str,
        storage: &Storage,
        chat_id: ChatId,
        file_id: FileId,
        caption: Option<&str>,
    ) -> Result<u64> {
        let mut inner = self.inner.lock().await;

        // id выдаём один раз на группу (только при первой вставке).
        if let Entry::Vacant(slot) = inner.entry(group_id.to_string()) {
            let id = storage.next_id().await?;
            slot.insert(Pending {
                id,
                chat_id,
                tag: None,
                caption: None,
                files: Vec::new(),
                tick: 0,
            });
        }

        let pending = inner
            .get_mut(group_id)
            .expect("запись только что вставлена или уже была");

        pending.tick += 1;
        pending.files.push(file_id);
        if let Some(caption) = caption {
            if pending.tag.is_none() {
                pending.tag = naming::tag_from(caption);
            }
            if pending.caption.is_none() {
                pending.caption = Some(caption.to_string());
            }
        }

        Ok(pending.tick)
    }

    /// Планирует сохранение альбома: если после `tick` новых фото не придёт,
    /// буфер сбросится в файлы и в чат уйдёт один отчёт.
    pub fn schedule_flush(
        self: &Arc<Self>,
        group_id: String,
        tick: u64,
        bot: Bot,
        storage: Arc<Storage>,
        chat_id: ChatId,
    ) {
        let albums = Arc::clone(self);
        tokio::spawn(async move {
            tokio::time::sleep(DEBOUNCE).await;

            match albums.flush_if_stale(&group_id, tick, &bot, &storage).await {
                Ok(Some(saved)) => {
                    let text = format!(
                        "✅ Альбом сохранён: {} фото\nметка: {}, id: {}",
                        saved.photos, saved.tag, saved.id
                    );
                    if let Err(error) = bot.send_message(chat_id, text).await {
                        tracing::warn!(error = %error, "Не удалось ответить про альбом");
                    }
                }
                // Альбом ещё пополняется — сбросит более поздний таймер.
                Ok(None) => {}
                Err(error) => tracing::error!(error = %error, "Не удалось сохранить альбом"),
            }
        });
    }

    /// Сохраняет альбом, если после `tick` не появилось новых фото.
    async fn flush_if_stale(
        &self,
        group_id: &str,
        tick: u64,
        bot: &Bot,
        storage: &Storage,
    ) -> Result<Option<SavedAlbum>> {
        let Some(pending) = self.take_if_stale(group_id, tick).await else {
            return Ok(None);
        };
        pending.save(bot, storage).await.map(Some)
    }

    /// Забирает альбом из буфера, если он «успокоился».
    async fn take_if_stale(&self, group_id: &str, tick: u64) -> Option<Pending> {
        let mut inner = self.inner.lock().await;
        let is_stale = inner
            .get(group_id)
            .is_some_and(|pending| pending.tick == tick);
        if is_stale {
            inner.remove(group_id)
        } else {
            None
        }
    }
}

impl Pending {
    /// Скачивает все фото альбома и сохраняет подпись.
    async fn save(&self, bot: &Bot, storage: &Storage) -> Result<SavedAlbum> {
        let tag = self
            .tag
            .clone()
            .unwrap_or_else(|| naming::FALLBACK_PHOTO.to_string());

        // Подпись — отдельным .txt с тем же id, что и файлы альбома.
        if let Some(caption) = &self.caption {
            let name = naming::file_name(&tag, self.id, None, "txt");
            storage.save_text(self.chat_id, &name, caption).await?;
        }

        for (index, file_id) in self.files.iter().enumerate() {
            let file = bot.get_file(file_id.clone()).await?;
            let name = naming::file_name(&tag, self.id, Some(index as u32 + 1), "jpg");
            let path = storage
                .save_photo(bot, &file.path, self.chat_id, &name)
                .await?;
            tracing::info!(path = %path.display(), "Сохранено фото альбома");
        }

        Ok(SavedAlbum {
            id: self.id,
            tag,
            photos: self.files.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("tg_album_{}_{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    /// Внутри одной группы id общий, у разных групп — разные.
    #[tokio::test]
    async fn album_shares_one_id_per_group() {
        let root = temp_root("ids");
        let storage = Storage::new(&root).unwrap();
        let albums = AlbumBuffer::new();

        let first = albums
            .push(
                "group-1",
                &storage,
                ChatId(1),
                FileId("a".into()),
                Some("Поездка в горы"),
            )
            .await
            .unwrap();
        let second = albums
            .push("group-1", &storage, ChatId(1), FileId("b".into()), None)
            .await
            .unwrap();
        let other = albums
            .push("group-2", &storage, ChatId(1), FileId("c".into()), None)
            .await
            .unwrap();

        assert_eq!(first, 1, "первый тик группы");
        assert_eq!(second, 2, "второй тик группы");
        assert_eq!(other, 1, "у другой группы свой отсчёт тиков");

        // id берём из следующих номеров: группа-1 получила id 1, группа-2 — id 2.
        assert_eq!(storage.next_id().await.unwrap(), 3, "выдано ровно два id");

        let _ = std::fs::remove_dir_all(&root);
    }
}
