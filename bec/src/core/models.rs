use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Автомобиль из каталога автосалона.
///
/// Структура описана один раз и используется на всех уровнях: трейт
/// [`crate::core::db::Database`] возвращает её из БД, а обработчики отдают
/// клиенту как JSON. Так SQL-схема (migrations) и API-контракт не расходятся.
#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct Car {
    pub id: i64,
    /// Номер лота на площадке, он же «номер записи» в каталоге (например, 0247).
    pub lot_number: String,
    pub brand: String,
    pub model: String,
    pub year: i64,
    /// Пробег в километрах.
    pub mileage: i64,
    pub engine: String,
    pub transmission: String,
    pub drive: String,
    /// Итоговая цена «под ключ» в рублях.
    pub price_rub: i64,
    /// Цена лота на аукционе в йенах (если известна).
    pub auction_price_yen: Option<i64>,
    pub auction_name: Option<String>,
    pub auction_lot: Option<String>,
    /// Аукционная оценка состояния, например "4.5".
    pub rating: Option<String>,
    /// Статус: `stock` | `transit` | `order` | `sold`.
    pub status: String,
    /// Срок доставки в днях для статусов `transit` / `order`.
    pub transit_days: Option<i64>,
    /// Город доставки для проданных авто.
    pub delivery_city: Option<String>,
    /// Обложка объявления (первое фото галереи).
    pub image_url: Option<String>,
    /// Галерея изображений в порядке показа (первое — обложка).
    ///
    /// Это не колонка таблицы `cars`, а данные из `car_images`, поэтому
    /// `#[sqlx(skip)]`: derive не ищет поле в строке, его заполняет
    /// `SqliteDb` отдельным запросом (см. `attach_images`).
    #[sqlx(skip)]
    pub images: Vec<String>,
    pub created_at: String,
    pub sold_at: Option<String>,
    /// Сколько дней заняла сделка от поступления до продажи.
    pub deal_duration_days: Option<i64>,
}

/// Сводная статистика по базе автомобилей.
///
/// Отдаётся одним запросом с COUNT(CASE ...), чтобы не гонять по сети
/// несколько отдельных подсчётов.
#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct Stats {
    /// Всего автомобилей в базе.
    pub total: i64,
    /// В наличии.
    pub in_stock: i64,
    /// В пути.
    pub in_transit: i64,
    /// Под заказ.
    pub on_order: i64,
    /// Продано.
    pub sold: i64,
}

/// Счётчик посещений сайта.
///
/// Доступен только админ-API: на публичных страницах и в публичном `/api/*`
/// этих чисел нет. `today` и `last_7_days` считаются по Владивостоку.
#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct VisitStats {
    /// Всего просмотров главной страницы сайта за всё время.
    pub total: i64,
    /// Просмотры за сегодня (VLAT).
    pub today: i64,
    /// Просмотры за последние 7 дней, включая сегодняшний.
    pub last_7_days: i64,
}

/// Контактная информация автосалона.
///
/// В таблице всегда одна строка (`id = 1`), поэтому отдельного списка нет.
#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct Contact {
    pub id: i64,
    pub phone_primary: String,
    pub phone_secondary: Option<String>,
    pub email: String,
    pub office_address: String,
    pub storage_address: String,
    pub working_hours: String,
    pub online_hours: String,
    pub company_name: String,
    pub inn: String,
    pub ogrn: String,
    pub telegram: Option<String>,
    pub whatsapp: Option<String>,
    pub youtube: Option<String>,
}

/// Смещение Владивостока (VLAT, UTC+10): все «живые» даты сайта — в нём.
const VLAT_OFFSET_HOURS: i64 = 10;

fn vlat_offset() -> Duration {
    Duration::hours(VLAT_OFFSET_HOURS)
}

/// Разбирает дату из SQLite (`YYYY-MM-DD HH:MM:SS`, время в UTC).
fn parse_utc(raw: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S"))
        .or_else(|_| NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S%.f"))
        .ok()
}

/// «сегодня в 09:40» / «вчера в 18:05» / «02.10.2026» — по времени Владивостока.
fn human_day(dt: NaiveDateTime) -> String {
    use chrono::Timelike;

    let local = dt + vlat_offset();
    let today = (Utc::now() + vlat_offset()).date_naive();
    let day = local.date();
    let time = format!("{:02}:{:02}", local.hour(), local.minute());

    if day == today {
        format!("сегодня в {time}")
    } else if Some(day) == today.pred_opt() {
        format!("вчера в {time}")
    } else {
        local.format("%d.%m.%Y").to_string()
    }
}

/// Текущее время Владивостока: «16.10.2026, 15:40 VLAT (UTC+10)».
pub fn now_vlat_display() -> String {
    (Utc::now() + vlat_offset())
        .format("%d.%m.%Y, %H:%M VLAT (UTC+10)")
        .to_string()
}

/// Разбивает целое на разряды пробелом: `1830000` → «1 830 000».
fn group_digits(value: i64) -> String {
    let digits = value.unsigned_abs().to_string();
    let len = digits.len();
    let mut out = String::with_capacity(len + len / 3);

    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (len - index).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(ch);
    }

    if value < 0 {
        format!("-{out}")
    } else {
        out
    }
}

/// Русское склонение слова «день»: 41 день, 33 дня, 37 дней.
fn plural_days(days: i64) -> String {
    let word = match (days % 10, days % 100) {
        (1, 11) => "дней",
        (1, _) => "день",
        (2..=4, 12..=14) => "дней",
        (2..=4, _) => "дня",
        _ => "дней",
    };
    format!("{days} {word}")
}

/// Значение опционального поля или длинное тире — чтобы шаблоны не знали про `Option`.
fn option_or_dash(value: &Option<String>) -> String {
    match value {
        Some(text) if !text.trim().is_empty() => text.clone(),
        _ => "—".to_string(),
    }
}

/// Оставляет в телефоне только цифры и ведущий «+» — формат для href="tel:".
fn tel_href(phone: &str) -> String {
    let mut out = String::with_capacity(phone.len());
    for (index, ch) in phone.chars().enumerate() {
        if ch.is_ascii_digit() {
            out.push(ch);
        } else if ch == '+' && index == 0 {
            out.push('+');
        }
    }
    out
}

impl Car {
    /// Название для заголовков: «Honda XR-V».
    pub fn title(&self) -> String {
        format!("{} {}", self.brand, self.model)
    }

    /// Заголовок карточки проданного авто: «Honda XR-V · 2023».
    pub fn title_with_year(&self) -> String {
        format!("{} {} · {}", self.brand, self.model, self.year)
    }

    /// Обложка: первое фото галереи, иначе колонка `image_url`, иначе пусто.
    pub fn cover(&self) -> String {
        self.images
            .first()
            .cloned()
            .or_else(|| self.image_url.clone())
            .unwrap_or_default()
    }

    /// «1 830 000 ₽».
    pub fn price_display(&self) -> String {
        format!("{} ₽", group_digits(self.price_rub))
    }

    /// «43 900 км».
    pub fn mileage_display(&self) -> String {
        format!("{} км", group_digits(self.mileage))
    }

    /// Строка характеристик в карточке каталога.
    pub fn specs_line(&self) -> String {
        format!(
            "{} · {} · {} · {} · {}",
            self.year,
            self.mileage_display(),
            self.engine,
            self.transmission,
            self.drive
        )
    }

    /// Подзаголовок главного акцента на первой странице.
    pub fn hero_sub(&self) -> String {
        format!(
            "{} · {} · {} год · привезён из Японии, растаможен, на площадке во Владивостоке",
            self.engine, self.drive, self.year
        )
    }

    /// CSS-класс плашки статуса (`stock` | `transit` | `order`).
    pub fn status_class(&self) -> &'static str {
        match self.status.as_str() {
            "transit" => "transit",
            "order" => "order",
            _ => "stock",
        }
    }

    /// Текст плашки статуса, например «В пути · 6 дней».
    pub fn status_label(&self) -> String {
        match (self.status.as_str(), self.transit_days) {
            ("transit", Some(days)) => format!("В пути · {}", plural_days(days)),
            ("transit", None) => "В пути".to_string(),
            ("order", Some(days)) => format!("Под заказ · ~{}", plural_days(days)),
            ("order", None) => "Под заказ".to_string(),
            _ => "В наличии".to_string(),
        }
    }

    /// Подпись под ценой в карточке.
    pub fn price_note(&self) -> &'static str {
        match self.status.as_str() {
            "transit" => "под выплату в пути",
            "order" => "расчёт «под ключ»",
            _ => "во Владивостоке",
        }
    }

    /// Аукционная оценка: «4.5 / A» или «—».
    pub fn rating_display(&self) -> String {
        match self.rating.as_deref() {
            Some(rating) if !rating.trim().is_empty() => format!("{rating} / A"),
            _ => "—".to_string(),
        }
    }

    /// Оценка только числом — для компактной строки метаданных.
    pub fn rating_short(&self) -> String {
        match self.rating.as_deref() {
            Some(rating) if !rating.trim().is_empty() => rating.to_string(),
            _ => "—".to_string(),
        }
    }

    pub fn auction_name_display(&self) -> String {
        option_or_dash(&self.auction_name)
    }

    pub fn auction_lot_display(&self) -> String {
        option_or_dash(&self.auction_lot)
    }

    /// «¥ 2 750 000» или пустая строка, если цена лота неизвестна.
    pub fn auction_price_display(&self) -> String {
        match self.auction_price_yen {
            Some(yen) => format!("¥ {}", group_digits(yen)),
            None => String::new(),
        }
    }

    /// Когда запись добавлена, по времени Владивостока.
    pub fn created_display(&self) -> String {
        match parse_utc(&self.created_at) {
            Some(dt) => human_day(dt),
            None => "—".to_string(),
        }
    }

    /// Дата продажи «12.10.2026» или «—».
    pub fn sold_date_display(&self) -> String {
        match self.sold_at.as_deref().and_then(parse_utc) {
            Some(dt) => (dt + vlat_offset()).format("%d.%m.%Y").to_string(),
            None => "—".to_string(),
        }
    }

    /// Срок сделки: «41 день» / «37 дней».
    pub fn deal_duration_display(&self) -> String {
        match self.deal_duration_days {
            Some(days) => plural_days(days),
            None => "—".to_string(),
        }
    }

    pub fn delivery_city_display(&self) -> String {
        option_or_dash(&self.delivery_city)
    }
}

impl Contact {
    /// Телефон в формате `tel:`.
    pub fn phone_primary_tel(&self) -> String {
        tel_href(&self.phone_primary)
    }

    pub fn phone_secondary_tel(&self) -> String {
        tel_href(self.phone_secondary.as_deref().unwrap_or_default())
    }

    pub fn phone_secondary_display(&self) -> String {
        self.phone_secondary.clone().unwrap_or_default()
    }

    pub fn has_secondary_phone(&self) -> bool {
        self.phone_secondary
            .as_deref()
            .is_some_and(|phone| !phone.trim().is_empty())
    }

    pub fn has_office_address(&self) -> bool {
        !self.office_address.trim().is_empty()
    }

    pub fn has_storage_address(&self) -> bool {
        !self.storage_address.trim().is_empty()
    }

    pub fn has_working_hours(&self) -> bool {
        !self.working_hours.trim().is_empty() || !self.online_hours.trim().is_empty()
    }

    pub fn has_email(&self) -> bool {
        !self.email.trim().is_empty()
    }

    pub fn has_requisites(&self) -> bool {
        !self.company_name.trim().is_empty()
            || !self.inn.trim().is_empty()
            || !self.ogrn.trim().is_empty()
    }

    /// Есть ли хоть одна строка с контактными деталями (адрес, часы, почта,
    /// реквизиты). Если нет — пустой блок с разделителем на сайте не нужен.
    pub fn has_details(&self) -> bool {
        self.has_office_address()
            || self.has_storage_address()
            || self.has_working_hours()
            || self.has_email()
            || self.has_requisites()
    }

    pub fn whatsapp_href(&self) -> String {
        self.whatsapp.clone().unwrap_or_else(|| "#".to_string())
    }

    pub fn telegram_href(&self) -> String {
        self.telegram.clone().unwrap_or_else(|| "#".to_string())
    }

    pub fn youtube_href(&self) -> String {
        self.youtube.clone().unwrap_or_else(|| "#".to_string())
    }

    pub fn has_whatsapp(&self) -> bool {
        has_contact_link(&self.whatsapp)
    }

    pub fn has_telegram(&self) -> bool {
        has_contact_link(&self.telegram)
    }

    pub fn has_youtube(&self) -> bool {
        has_contact_link(&self.youtube)
    }
}

/// Ссылка на мессенджер задана и не пустая (пустая строка не должна
/// превращаться в кликабельную ссылку-заглушку).
fn has_contact_link(value: &Option<String>) -> bool {
    value.as_deref().is_some_and(|link| !link.trim().is_empty())
}

/// Набор фильтров каталога для серверного рендеринга страниц на htmx.
///
/// JSON-эндпоинт `/api/cars` работает с двумя простыми параметрами, а страница
/// отдаёт ещё поиск и сортировку — поэтому здесь отдельная структура, а не
/// разрастание сигнатуры [`crate::core::db::Database::list_cars`].
#[derive(Debug, Clone, Default)]
pub struct CarFilter {
    /// `stock` | `transit` | `order`; `None` — всё, кроме проданных.
    pub status: Option<String>,
    /// Фильтр по марке (без учёта регистра).
    pub brand: Option<String>,
    /// Поиск по марке, модели и номеру лота.
    pub query: Option<String>,
    /// Сортировка: `new` (по умолчанию), `price-asc`, `price-desc`, `mileage`.
    pub sort: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

/// Допустимые статусы автомобиля.
///
/// Один список на весь проект: и валидация админ-API, и фильтры каталога
/// проверяют значения по нему.
pub const CAR_STATUSES: [&str; 4] = ["stock", "transit", "order", "sold"];

/// Данные для создания автомобиля через админ-API (`POST /api/admin/cars`).
///
/// Обязательные поля идут без `Option` — их отсутствие отсекает десериализация
/// ещё до обработчика. Необязательные помечены `#[serde(default)]`, чтобы
/// клиенту не приходилось присылать весь набор колонок.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct NewCar {
    /// Номер лота. Если не задан или пуст — сервер присвоит следующий свободный.
    #[serde(default)]
    pub lot_number: Option<String>,
    pub brand: String,
    pub model: String,
    pub year: i64,
    pub mileage: i64,
    pub engine: String,
    pub transmission: String,
    pub drive: String,
    pub price_rub: i64,
    #[serde(default)]
    pub auction_price_yen: Option<i64>,
    #[serde(default)]
    pub auction_name: Option<String>,
    #[serde(default)]
    pub auction_lot: Option<String>,
    #[serde(default)]
    pub rating: Option<String>,
    /// Статус: `stock` | `transit` | `order` | `sold`. По умолчанию `stock`.
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub transit_days: Option<i64>,
    #[serde(default)]
    pub delivery_city: Option<String>,
    /// Обложка объявления (если задана без галереи).
    #[serde(default)]
    pub image_url: Option<String>,
}

/// Смена статуса автомобиля (`PATCH /api/admin/cars/{id}/status`).
///
/// Обязателен только `status`; `transit_days` и `delivery_city` — уточнения для
/// статусов «в пути»/«под заказ» и «продано» соответственно.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct StatusUpdate {
    pub status: String,
    #[serde(default)]
    pub transit_days: Option<i64>,
    #[serde(default)]
    pub delivery_city: Option<String>,
}
