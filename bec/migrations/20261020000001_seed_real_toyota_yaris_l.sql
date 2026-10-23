-- Реальное объявление из real_data/Toyota_Yaris_L/Toyota_6.txt
-- (Toyota Yaris L, июль 2022, 33 200 км, 2WD, 1.5 (112 л.с.), 1 398 000 ₽ «под ключ»).
--
-- Отдельной миграцией, а не правкой предыдущих: они уже применены к dev_sqlite.db,
-- и sqlx сверяет контрольную сумму — изменённый файл уронил бы старт сервера.
--
-- Чего нет в объявлении, восстановлено по модели (как с Honda XR-V и Audi A3):
--   transmission = CVT: Yaris L с двигателем 1.5 идёт с бесступенчатым вариатором.
-- Номер лота 0250 — следующий свободный после 0248 (Honda) и 0249 (Audi).
-- Обложка (image_url) — первый кадр реальной галереи из static/images.
INSERT OR IGNORE INTO cars (lot_number, brand, model, year, mileage, engine, transmission, drive, price_rub, auction_price_yen, auction_name, auction_lot, rating, status, transit_days, delivery_city, image_url, created_at, sold_at, deal_duration_days)
VALUES
    ('0250', 'Toyota', 'Yaris L', 2022, 33200, '1.5 (112 л.с.)', 'CVT', '2WD', 1398000, NULL, NULL, NULL, NULL, 'stock', NULL, NULL, '/images/toyota-yaris-l/toyota-yaris-l-1.jpg', datetime('now'), NULL, NULL);

-- Галерея Toyota Yaris L (лот 0250). Порядок как в альбоме из Telegram:
-- 0 — обложка, далее экстерьер, салон и детали.
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 0, '/images/toyota-yaris-l/toyota-yaris-l-1.jpg' FROM cars WHERE lot_number = '0250';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 1, '/images/toyota-yaris-l/toyota-yaris-l-2.jpg' FROM cars WHERE lot_number = '0250';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 2, '/images/toyota-yaris-l/toyota-yaris-l-3.jpg' FROM cars WHERE lot_number = '0250';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 3, '/images/toyota-yaris-l/toyota-yaris-l-4.jpg' FROM cars WHERE lot_number = '0250';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 4, '/images/toyota-yaris-l/toyota-yaris-l-5.jpg' FROM cars WHERE lot_number = '0250';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 5, '/images/toyota-yaris-l/toyota-yaris-l-6.jpg' FROM cars WHERE lot_number = '0250';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 6, '/images/toyota-yaris-l/toyota-yaris-l-7.jpg' FROM cars WHERE lot_number = '0250';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 7, '/images/toyota-yaris-l/toyota-yaris-l-8.jpg' FROM cars WHERE lot_number = '0250';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 8, '/images/toyota-yaris-l/toyota-yaris-l-9.jpg' FROM cars WHERE lot_number = '0250';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 9, '/images/toyota-yaris-l/toyota-yaris-l-10.jpg' FROM cars WHERE lot_number = '0250';
