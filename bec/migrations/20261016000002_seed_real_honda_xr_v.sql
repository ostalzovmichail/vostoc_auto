-- Реальное объявление из real_data/honda_xr-v-2023/description.txt
-- (Honda XR-V, сентябрь 2023, 43 900 км, 2WD, 1.5 (124 л.с.), 1 830 000 ₽ «под ключ»).
--
-- Отдельной миграцией, а не правкой предыдущей: та уже применена к dev_sqlite.db,
-- и sqlx сверяет контрольную сумму — изменённый файл уронил бы старт сервера.
--
-- Спецификации, которых нет в объявлении, восстановлены по модели:
--   transmission = CVT (XR-V с 1.5 идёт с вариатором), drive = 2WD.
-- Номер лота 0248 — следующий свободный; created_at = сейчас, поэтому авто
-- становится самым свежим и попадает в блок «Только что в базе».
-- Обложка (image_url) — первый кадр реальной галереи из static/images.
INSERT OR IGNORE INTO cars (lot_number, brand, model, year, mileage, engine, transmission, drive, price_rub, auction_price_yen, auction_name, auction_lot, rating, status, transit_days, delivery_city, image_url, created_at, sold_at, deal_duration_days)
VALUES
    ('0248', 'Honda', 'XR-V', 2023, 43900, '1.5 (124 л.с.)', 'CVT', '2WD', 1830000, NULL, NULL, NULL, NULL, 'stock', NULL, NULL, '/images/honda-xr-v-2023/honda-xr-v-2023-1.jpeg', datetime('now'), NULL, NULL);
