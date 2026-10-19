-- Реальное объявление из real_data/audi_a3/Audi_2.txt
-- (Audi A3, ноябрь 2021, 55 200 км, 2WD, 1.4T (150 л.с.), 1 940 000 ₽ «под ключ»).
--
-- Отдельной миграцией, а не правкой предыдущих: они уже применены к dev_sqlite.db,
-- и sqlx сверяет контрольную сумму — изменённый файл уронил бы старт сервера.
--
-- Чего нет в объявлении, восстановлено по модели (как с Honda XR-V):
--   transmission = S tronic: A3 35 TFSI с двигателем 1.4 идёт с 7-ступ. DSG.
-- Номер лота 0249 — следующий свободный; created_at = сейчас, поэтому авто
-- становится самым свежим и попадает в блок «Только что в базе».
-- Обложка (image_url) — первый кадр галереи.
INSERT OR IGNORE INTO cars (lot_number, brand, model, year, mileage, engine, transmission, drive, price_rub, auction_price_yen, auction_name, auction_lot, rating, status, transit_days, delivery_city, image_url, created_at, sold_at, deal_duration_days)
VALUES
    ('0249', 'Audi', 'A3', 2021, 55200, '1.4T (150 л.с.)', 'S tronic', '2WD', 1940000, NULL, NULL, NULL, NULL, 'stock', NULL, NULL, '/images/audi-a3/audi-a3-1.jpg', datetime('now'), NULL, NULL);

-- Галерея Audi A3 (лот 0249). Порядок как в альбоме из Telegram:
-- 0 — обложка (передний «три четверти»), далее экстерьер, салон и детали.
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 0, '/images/audi-a3/audi-a3-1.jpg' FROM cars WHERE lot_number = '0249';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 1, '/images/audi-a3/audi-a3-2.jpg' FROM cars WHERE lot_number = '0249';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 2, '/images/audi-a3/audi-a3-3.jpg' FROM cars WHERE lot_number = '0249';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 3, '/images/audi-a3/audi-a3-4.jpg' FROM cars WHERE lot_number = '0249';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 4, '/images/audi-a3/audi-a3-5.jpg' FROM cars WHERE lot_number = '0249';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 5, '/images/audi-a3/audi-a3-6.jpg' FROM cars WHERE lot_number = '0249';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 6, '/images/audi-a3/audi-a3-7.jpg' FROM cars WHERE lot_number = '0249';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 7, '/images/audi-a3/audi-a3-8.jpg' FROM cars WHERE lot_number = '0249';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 8, '/images/audi-a3/audi-a3-9.jpg' FROM cars WHERE lot_number = '0249';
