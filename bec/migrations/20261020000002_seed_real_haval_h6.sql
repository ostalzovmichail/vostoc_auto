-- Реальное объявление из real_data/Haval_H6/Haval_8.txt
-- (Haval H6, октябрь 2021, 55 000 км, 2WD, 1.5T (143 л.с.), 1 350 000 ₽ «под ключ»).
--
-- Отдельной миграцией, а не правкой предыдущих: они уже применены к dev_sqlite.db,
-- и sqlx сверяет контрольную сумму — изменённый файл уронил бы старт сервера.
--
-- Чего нет в объявлении, восстановлено по модели (как с Honda XR-V и Audi A3):
--   transmission = 7DCT: Haval H6 с турбомотором 1.5 идёт с 7-ступенчатым роботом.
-- Номер лота 0251 — следующий свободный; created_at = сейчас.
-- Обложка (image_url) — первый кадр реальной галереи из static/images.
INSERT OR IGNORE INTO cars (lot_number, brand, model, year, mileage, engine, transmission, drive, price_rub, auction_price_yen, auction_name, auction_lot, rating, status, transit_days, delivery_city, image_url, created_at, sold_at, deal_duration_days)
VALUES
    ('0251', 'Haval', 'H6', 2021, 55000, '1.5T (143 л.с.)', '7DCT', '2WD', 1350000, NULL, NULL, NULL, NULL, 'stock', NULL, NULL, '/images/haval-h6/haval-h6-1.jpg', datetime('now'), NULL, NULL);

-- Галерея Haval H6 (лот 0251). Порядок как в альбоме из Telegram:
-- 0 — обложка, далее экстерьер, салон и детали.
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 0, '/images/haval-h6/haval-h6-1.jpg' FROM cars WHERE lot_number = '0251';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 1, '/images/haval-h6/haval-h6-2.jpg' FROM cars WHERE lot_number = '0251';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 2, '/images/haval-h6/haval-h6-3.jpg' FROM cars WHERE lot_number = '0251';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 3, '/images/haval-h6/haval-h6-4.jpg' FROM cars WHERE lot_number = '0251';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 4, '/images/haval-h6/haval-h6-5.jpg' FROM cars WHERE lot_number = '0251';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 5, '/images/haval-h6/haval-h6-6.jpg' FROM cars WHERE lot_number = '0251';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 6, '/images/haval-h6/haval-h6-7.jpg' FROM cars WHERE lot_number = '0251';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 7, '/images/haval-h6/haval-h6-8.jpg' FROM cars WHERE lot_number = '0251';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 8, '/images/haval-h6/haval-h6-9.jpg' FROM cars WHERE lot_number = '0251';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 9, '/images/haval-h6/haval-h6-10.jpg' FROM cars WHERE lot_number = '0251';
