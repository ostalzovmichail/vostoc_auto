-- Реальное объявление из real_data/Volkswagen_Tacqua/Volkswagen_10.txt
-- (Volkswagen Tacqua, декабрь 2021, 55 400 км, 2WD, 1.5 (113 л.с.), 1 550 000 ₽ «под ключ»).
--
-- Отдельной миграцией, а не правкой предыдущих: они уже применены к dev_sqlite.db,
-- и sqlx сверяет контрольную сумму — изменённый файл уронил бы старт сервера.
--
-- Чего нет в объявлении, восстановлено по модели (как с Honda XR-V и Audi A3):
--   transmission = 6AT: Tacqua с атмосферником 1.5 идёт с 6-ступ. автоматом Aisin.
-- Номер лота 0252 — следующий свободный; created_at = сейчас.
-- Обложка (image_url) — первый кадр реальной галереи из static/images.
INSERT OR IGNORE INTO cars (lot_number, brand, model, year, mileage, engine, transmission, drive, price_rub, auction_price_yen, auction_name, auction_lot, rating, status, transit_days, delivery_city, image_url, created_at, sold_at, deal_duration_days)
VALUES
    ('0252', 'Volkswagen', 'Tacqua', 2021, 55400, '1.5 (113 л.с.)', '6AT', '2WD', 1550000, NULL, NULL, NULL, NULL, 'stock', NULL, NULL, '/images/volkswagen-tacqua/volkswagen-tacqua-1.jpg', datetime('now'), NULL, NULL);

-- Галерея Volkswagen Tacqua (лот 0252). Порядок как в альбоме из Telegram:
-- 0 — обложка, далее экстерьер, салон и детали.
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 0, '/images/volkswagen-tacqua/volkswagen-tacqua-1.jpg' FROM cars WHERE lot_number = '0252';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 1, '/images/volkswagen-tacqua/volkswagen-tacqua-2.jpg' FROM cars WHERE lot_number = '0252';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 2, '/images/volkswagen-tacqua/volkswagen-tacqua-3.jpg' FROM cars WHERE lot_number = '0252';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 3, '/images/volkswagen-tacqua/volkswagen-tacqua-4.jpg' FROM cars WHERE lot_number = '0252';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 4, '/images/volkswagen-tacqua/volkswagen-tacqua-5.jpg' FROM cars WHERE lot_number = '0252';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 5, '/images/volkswagen-tacqua/volkswagen-tacqua-6.jpg' FROM cars WHERE lot_number = '0252';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 6, '/images/volkswagen-tacqua/volkswagen-tacqua-7.jpg' FROM cars WHERE lot_number = '0252';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 7, '/images/volkswagen-tacqua/volkswagen-tacqua-8.jpg' FROM cars WHERE lot_number = '0252';
INSERT OR IGNORE INTO car_images (car_id, position, url)
SELECT id, 8, '/images/volkswagen-tacqua/volkswagen-tacqua-9.jpg' FROM cars WHERE lot_number = '0252';
