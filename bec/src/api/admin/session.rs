//! Сессия администратора: подписанная cookie и защита от перебора токена.
//!
//! Токен администратора (`ADMIN_TOKEN`) — единственный секрет. Чтобы не
//! таскать его в каждом запросе и не хранить в localStorage, после входа
//! браузер получает HttpOnly-cookie с подписью HMAC-SHA256:
//!
//! ```text
//! admin_session = <expires_unix>.<hex(hmac_sha256(ADMIN_TOKEN, expires))>
//! ```
//!
//! Подделать значение без токена нельзя, а смена `ADMIN_TOKEN` автоматически
//! инвалидирует все выданные сессии.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::http::{header, HeaderMap};
use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Имя cookie сессии.
pub const COOKIE_NAME: &str = "bec_admin";
/// Срок жизни сессии — 12 часов.
pub const SESSION_TTL_SECS: i64 = 12 * 60 * 60;

/// Сколько неудачных попыток входа допускается в окне.
const MAX_FAILURES: u32 = 8;
/// Длина окна для счётчика неудачных попыток.
const FAILURE_WINDOW: Duration = Duration::from_secs(10 * 60);

/// Подпись сессии: HMAC-SHA256 от времени истечения.
fn sign(secret: &str, expires: i64) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC принимает ключ любой длины");
    mac.update(expires.to_string().as_bytes());
    to_hex(&mac.finalize().into_bytes())
}

/// Выдаёт новую cookie: значение и время истечения (unix).
pub fn issue(secret: &str, now: i64) -> (String, i64) {
    let expires = now + SESSION_TTL_SECS;
    (format!("{expires}.{}", sign(secret, expires)), expires)
}

/// Проверяет формат, подпись и срок действия cookie.
pub fn verify(secret: &str, value: &str, now: i64) -> bool {
    let Some((expires_raw, signature)) = value.split_once('.') else {
        return false;
    };
    let Ok(expires) = expires_raw.parse::<i64>() else {
        return false;
    };
    if expires <= now {
        return false;
    }

    let expected = sign(secret, expires);
    // Сравнение подписей — побайтовое и без раннего выхода.
    constant_time_eq(signature.as_bytes(), expected.as_bytes())
}

/// Заголовок `Set-Cookie` для выданной сессии.
pub fn set_cookie(value: &str, secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!(
        "{COOKIE_NAME}={value}; Path=/admin; HttpOnly; SameSite=Strict; Max-Age={SESSION_TTL_SECS}{secure_flag}"
    )
}

/// Заголовок `Set-Cookie`, удаляющий сессию.
pub fn clear_cookie(secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!("{COOKIE_NAME}=; Path=/admin; HttpOnly; SameSite=Strict; Max-Age=0{secure_flag}")
}

/// Достаёт значение сессионной cookie из заголовков запроса.
pub fn cookie_value(headers: &HeaderMap) -> Option<String> {
    for value in headers.get_all(header::COOKIE) {
        let Ok(raw) = value.to_str() else { continue };
        for pair in raw.split(';') {
            let pair = pair.trim();
            let Some((name, value)) = pair.split_once('=') else {
                continue;
            };
            if name.trim() == COOKIE_NAME {
                return Some(value.trim().to_string());
            }
        }
    }
    None
}

/// Текущее время в unix-секундах.
pub fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

/// Побайтовое сравнение за постоянное время.
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

/// Шестнадцатеричная запись байт (без зависимости от `hex`).
fn to_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(DIGITS[(byte >> 4) as usize] as char);
        out.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Простое ограничение попыток входа по IP.
///
/// Хранится в памяти процесса: для одного сервера этого достаточно, а при
/// рестарте счётчик сбрасывается — не критично, потому что основная защита
/// всё равно в длине токена.
#[derive(Default)]
pub struct LoginLimiter {
    windows: Mutex<HashMap<String, FailureWindow>>,
}

struct FailureWindow {
    failures: u32,
    reset_at: Instant,
}

impl LoginLimiter {
    /// Заблокирован ли ключ (IP) на текущий момент.
    pub fn is_blocked(&self, key: &str) -> bool {
        let Ok(mut windows) = self.windows.lock() else {
            return false;
        };
        let now = Instant::now();
        match windows.get_mut(key) {
            Some(window) if now >= window.reset_at => {
                windows.remove(key);
                false
            }
            Some(window) => window.failures >= MAX_FAILURES,
            None => false,
        }
    }

    /// Учитывает неудачную попытку.
    pub fn record_failure(&self, key: &str) {
        let Ok(mut windows) = self.windows.lock() else {
            return;
        };
        let now = Instant::now();
        let window = windows.entry(key.to_string()).or_insert(FailureWindow {
            failures: 0,
            reset_at: now + FAILURE_WINDOW,
        });
        if now >= window.reset_at {
            window.failures = 0;
            window.reset_at = now + FAILURE_WINDOW;
        }
        window.failures = window.failures.saturating_add(1);
    }

    /// Сбрасывает счётчик после успешного входа.
    pub fn reset(&self, key: &str) {
        if let Ok(mut windows) = self.windows.lock() {
            windows.remove(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_roundtrip() {
        let secret = "secret";
        let now = 1_000;
        let (value, expires) = issue(secret, now);
        assert_eq!(expires, now + SESSION_TTL_SECS);
        assert!(verify(secret, &value, now));
    }

    #[test]
    fn session_rejects_tampering() {
        let secret = "secret";
        let (value, _) = issue(secret, 1_000);
        let (expires, _) = value.split_once('.').unwrap();
        assert!(!verify(secret, &format!("{expires}.deadbeef"), 1_000));
        assert!(!verify("other-secret", &value, 1_000));
        assert!(!verify(secret, "garbage", 1_000));
    }

    #[test]
    fn session_expires() {
        let secret = "secret";
        let now = 1_000;
        let (value, expires) = issue(secret, now);
        assert!(!verify(secret, &value, expires));
    }

    #[test]
    fn cookie_is_parsed() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            "foo=1; bec_admin=123.abc; bar=2".parse().unwrap(),
        );
        assert_eq!(cookie_value(&headers).as_deref(), Some("123.abc"));
    }
}
