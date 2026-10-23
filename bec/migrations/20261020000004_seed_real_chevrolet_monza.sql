-- Реальное объявление из real_data/Chevrolet_Monza/Chevrolet_12.txt
-- (Chevrolet Monza, декабрь 2022, 32 600 км, 2WD, 1.5 (109 л.с.), 1 298 000 ₽ «под ключ»).
--
-- Отдельной миграцией, а не правкой предыдущих: они уже применены к dev_sqlite.db,
-- и sqlx сверяет контрольную сумму — изменённый файл уронил бы старт сервера.
--
-- Чего нет в объявлении, восстановлено по модели (как с Honda XR-V и Audi A3):
--   transmission = 6AT: Monza 320 с двигателем 1.5 идёт с 6-ступ. гидроавтоматом.
-- Номер лота 0253 — следующий свободный; created_at = сейчас, поэтому авто
-- становится самым свежим и попадает в блок «Только что в базе».
-- Обложка (image_url) — первый кадр реальной галереи из static/images.
INSERT OR IGNORE INTO cars (lot_number, brand, model, year, mileage, engine, transmission, drive, price_rub, auction_price_yen, auction_name, auction_lot, rating, status, transit_days, delivery_city, image_url, created_at, sold_at, deal_duration_days)
VALUES
    ('0253', 'Chevrolet', 'Monza', 2022, 32600, '1.5 (109 л.с.)', '6AT', '2WD', 1298000, NULL, NULL, NULL, NULL, 'stock', NULL, NULL, '/images/chevrolet-monza/chevrolet-monza-1.jpg', datetime('now'), NULL, NULL);

-- Галерея Chevrolet Monza (лот 0253). Порядок как в альбоме из Telegram:
-- 0 — обложка, далее экстерьер, салон и детали.
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 0, '/images/chevrolet-monza/chevrolet-monza-1.jpg' FROM cars WHERE lot_number = '0253';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 1, '/images/chevrolet-monza/chevrolet-monza-2.jpg' FROM cars WHERE lot_number = '0253';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 2, '/images/chevrolet-monza/chevrolet-monza-3.jpg' FROM cars WHERE lot_number = '0253';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 3, '/images/chevrolet-monza/chevrolet-monza-4.jpg' FROM cars WHERE lot_number = '0253';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 4, '/images/chevrolet-monza/chevrolet-monza-5.jpg' FROM cars WHERE lot_number = '0253';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 5, '/images/chevrolet-monza/chevrolet-monza-6.jpg' FROM cars WHERE lot_number = '0253';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 6, '/images/chevrolet-monza/chevrolet-monza-7.jpg' FROM cars WHERE lot_number = '0253';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 7, '/images/chevrolet-monza/chevrolet-monza-8.jpg' FROM cars WHERE lot_number = '0253';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 8, '/images/chevrolet-monza/chevrolet-monza-9.jpg' FROM cars WHERE lot_number = '0253';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 9, '/images/chevrolet-monza/chevrolet-monza-10.jpg' FROM cars WHERE lot_number = '0253';
