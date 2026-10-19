-- Таблица автомобилей
CREATE TABLE IF NOT EXISTS cars (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    lot_number TEXT NOT NULL UNIQUE,
    brand TEXT NOT NULL,
    model TEXT NOT NULL,
    year INTEGER NOT NULL,
    mileage INTEGER NOT NULL,
    engine TEXT NOT NULL,
    transmission TEXT NOT NULL,
    drive TEXT NOT NULL,
    price_rub INTEGER NOT NULL,
    auction_price_yen INTEGER,
    auction_name TEXT,
    auction_lot TEXT,
    rating TEXT,
    status TEXT NOT NULL CHECK(status IN ('stock', 'transit', 'order', 'sold')),
    transit_days INTEGER,
    delivery_city TEXT,
    image_url TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    sold_at DATETIME,
    deal_duration_days INTEGER
);

-- Индексы для быстрых запросов
CREATE INDEX IF NOT EXISTS idx_cars_status ON cars(status);
CREATE INDEX IF NOT EXISTS idx_cars_created_at ON cars(created_at DESC);

-- Контактная информация
CREATE TABLE IF NOT EXISTS contacts (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    phone_primary TEXT NOT NULL,
    phone_secondary TEXT,
    email TEXT NOT NULL,
    office_address TEXT NOT NULL,
    storage_address TEXT NOT NULL,
    working_hours TEXT NOT NULL,
    online_hours TEXT NOT NULL,
    company_name TEXT NOT NULL,
    inn TEXT NOT NULL,
    ogrn TEXT NOT NULL,
    telegram TEXT,
    whatsapp TEXT,
    youtube TEXT
);

-- Контактная информация.
-- Заполняем только реальные данные из real_data: телефон для связи.
-- Адрес, реквизиты и мессенджеры пока неизвестны — оставляем пустыми,
-- чтобы сайт не показывал вымышленные данные.
INSERT OR IGNORE INTO contacts (id, phone_primary, phone_secondary, email, office_address, storage_address, working_hours, online_hours, company_name, inn, ogrn, telegram, whatsapp, youtube)
VALUES (
    1,
    '89996832070',
    NULL,
    '',
    '',
    '',
    '',
    '',
    '',
    '',
    '',
    NULL,
    NULL,
    NULL
);

-- Демонстрационные автомобили здесь не создаются: каталог наполняется
-- только реальными объявлениями из отдельных миграций (Honda XR-V, Audi A3).
