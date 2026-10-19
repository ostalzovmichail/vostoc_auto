//! Правила именования сохранённых файлов.
//!
//! Имя файла: `<метка>_<id>[_<номер>].<расширение>`, где
//! * **метка** — первое слово первой строки текста (или подписи к фото);
//! * **id** — сквозной номер, который бот присваивает при сохранении;
//! * **номер** — только у альбомов: фото одной группы различаются им.

/// Запасная метка, если в тексте нет подходящего первого слова.
pub const FALLBACK_TEXT: &str = "text";
/// Запасная метка для фото без подписи.
pub const FALLBACK_PHOTO: &str = "photo";

/// Максимальная длина метки в имени файла.
const MAX_TAG_LEN: usize = 40;

/// Возвращает первое слово первой строки — метку для имени файла.
///
/// Из слова убираются символы, недопустимые или неудобные в имени файла
/// (остаются буквы, цифры, `-` и `_`). Если после очистки ничего не осталось,
/// возвращается `None` — вызывающий подставит запасную метку.
pub fn tag_from(text: &str) -> Option<String> {
    let first_line = text.lines().next().unwrap_or("");
    let word = first_line.split_whitespace().next()?;

    let cleaned: String = word
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '-' | '_'))
        .take(MAX_TAG_LEN)
        .collect();

    (!cleaned.is_empty()).then_some(cleaned)
}

/// Собирает имя файла из метки, id и необязательного номера в группе.
pub fn file_name(tag: &str, id: u64, index: Option<u32>, ext: &str) -> String {
    match index {
        Some(index) => format!("{tag}_{id}_{index}.{ext}"),
        None => format!("{tag}_{id}.{ext}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_is_first_word_of_first_line() {
        // Запятая — не часть слова, в имя файла не попадает.
        assert_eq!(
            tag_from("Привет, мир!\nвторая строка"),
            Some("Привет".into())
        );
        assert_eq!(tag_from("первая\nвторая"), Some("первая".into()));
    }

    #[test]
    fn tag_none_without_word() {
        assert_eq!(tag_from(""), None);
        assert_eq!(tag_from("!!! ???"), None);
        assert_eq!(tag_from("\nтекст"), None); // первая строка пуста
    }

    #[test]
    fn tag_keeps_digits_and_dash() {
        assert_eq!(tag_from("Audi-Q5 — 2023"), Some("Audi-Q5".into()));
    }

    #[test]
    fn file_name_with_and_without_index() {
        assert_eq!(file_name("Привет", 7, None, "txt"), "Привет_7.txt");
        assert_eq!(file_name("photo", 12, Some(3), "jpg"), "photo_12_3.jpg");
    }
}
