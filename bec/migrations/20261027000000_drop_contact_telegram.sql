-- Telegram больше не используется как канал связи: бот удалён из проекта,
-- а ссылка на Telegram убрана из блока контактов и публичного API.
-- SQLite 3.35+ поддерживает DROP COLUMN; версия, встроенная в sqlx, новее.
ALTER TABLE contacts DROP COLUMN telegram;
