-- Галерея изображений автомобиля.
--
-- Отдельная таблица, а не колонка в `cars`: у объявления бывает несколько
-- фото, и их порядок важен (первое — обложка). Одним TEXT-полем с разделителем
-- пришлось бы вручную парсить строку; здесь порядок задаёт `position`.
CREATE TABLE IF NOT EXISTS car_images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    car_id INTEGER NOT NULL REFERENCES cars(id) ON DELETE CASCADE,
    -- Позиция в галерее: 0 — обложка.
    position INTEGER NOT NULL,
    -- Путь относительный (без хоста): файлы отдаёт сам бэкенд из ./static
    -- по маршруту /images, а фронтенд сам подставляет адрес API. Так в базе
    -- не «зашит» ни localhost, ни прод-домен.
    url TEXT NOT NULL,
    -- Защита от дублей: у одного авто на одной позиции ровно одно фото.
    UNIQUE (car_id, position)
);

CREATE INDEX IF NOT EXISTS idx_car_images_car ON car_images(car_id, position);

-- Реальные фото Honda XR-V (лот 0248). Порядок выбран по ракурсам:
-- 1 — обложка (передний «три четверти»), далее салон/детали и корма.
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 0, '/images/honda-xr-v-2023/honda-xr-v-2023-1.jpeg' FROM cars WHERE lot_number = '0248';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 1, '/images/honda-xr-v-2023/honda-xr-v-2023-2.jpeg' FROM cars WHERE lot_number = '0248';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 2, '/images/honda-xr-v-2023/honda-xr-v-2023-3.jpeg' FROM cars WHERE lot_number = '0248';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 3, '/images/honda-xr-v-2023/honda-xr-v-2023-4.jpeg' FROM cars WHERE lot_number = '0248';

-- Обложка объявления = первое фото галереи (раньше тут был внешний placeholder).
UPDATE cars
   SET image_url = '/images/honda-xr-v-2023/honda-xr-v-2023-1.jpeg'
 WHERE lot_number = '0248';
