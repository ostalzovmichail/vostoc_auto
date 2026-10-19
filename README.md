# Берег

Сайт автосалона «Берег»: каталог автомобилей из Японии, серверный фронтенд на htmx,
админ-приложение на Flutter и вспомогательный Telegram-бот.

Репозиторий — монорепозиторий из нескольких независимых частей. Каждая часть
собирается и тестируется отдельно; подробности — в её собственном README.

---

## Состав проекта

```
bereg/
├── bec/                 # Бэкенд + публичный сайт (Rust, axum)
│   ├── src/             # API, обработчики, БД, конфигурация
│   ├── migrations/      # SQL-миграции (sqlx), включая реальные данные
│   ├── templates/       # Askama-шаблоны HTML-фрагментов
│   ├── frontend/        # index.html, CSS, JS, htmx — раздаётся на /ui/
│   ├── static/images/   # Фотографии автомобилей
│   ├── .env.example     # Шаблон конфигурации
│   └── README.md        # Подробно про API, конфиг и запуск
│
├── admin_app/           # Админ-приложение (Flutter, Android/iOS/Web/Desktop)
│   └── AGENTS.md        # Архитектура, конвенции, команды
│
├── TG/                  # Telegram-бот (Rust, teloxide) — сохраняет тексты и фото
│   └── README.md
│
├── real_data/           # Исходные реальные данные (Honda XR-V, Audi A3)
├── disain/              # Макет-референс «ВОСТОК АВТО» (не используется сервером)
├── .gitignore           # Общие правила игнорирования
├── ТЗ_Миша_2026-10-20.md
└── Задание_Миша_2026-10-20.md
```

### Что где работает

| Часть | Технологии | Назначение |
|---|---|---|
| `bec` | Rust, axum, SQLite, sqlx, askama, htmx | Публичный сайт, JSON API, админ-API, раздача фото |
| `admin_app` | Flutter, Riverpod 3, Material 3 | Управление каталогом: авто, статусы, фото |
| `TG` | Rust, teloxide | Сохраняет присланные боту сообщения и фото на диск |

---

## Быстрый старт (разработка)

### 1. Бэкенд и сайт

```bash
cd bec
cp .env.example .env.dev      # при необходимости заполните ADMIN_TOKEN
cargo run
```

Открыть: **<http://localhost:8080/ui/>**

- Публичный API: `http://localhost:8080/api/*`
- Swagger UI (только в dev): `http://localhost:8080/swagger-ui`
- Dev-база: `bec/dev_sqlite.db` (создаётся миграциями при старте)
- Логи: `bec/logs/YYYY/MM/DD/`

### 2. Админ-приложение

```bash
cd admin_app
flutter pub get
dart run build_runner build --delete-conflicting-outputs
flutter run
```

В настройках приложения указать адрес сервера (`http://127.0.0.1:8080`) и
`ADMIN_TOKEN` из `bec/.env.dev`. Для Android-эмулятора хост — `10.0.2.2`.

### 3. Telegram-бот

```bash
cd TG
cp .env.example .env          # вставьте токен от @BotFather
cargo run
```

---

## Конфигурация бэкенда

Значения читаются из окружения; файлы подключаются в порядке
`.env.{APP_ENV}` → `.env` (системное окружение имеет приоритет).

| Переменная | Обязательна | Назначение |
|---|---|---|
| `APP_ENV` | нет | `dev` (по умолчанию) или `prod` |
| `DATABASE_URL` | да | Строка подключения SQLite (`sqlite:./dev_sqlite.db`) |
| `SERVER_HOST` | да | Адрес привязки (`127.0.0.1` в dev, `0.0.0.0` в prod) |
| `SERVER_PORT` | да | Порт (по умолчанию `8080`) |
| `START_MESSAGE` | да | Сообщение при старте |
| `CORS_ALLOWED_ORIGINS` | нет | Разрешённые origin'ы (только prod) |
| `ADMIN_TOKEN` | нет | Токен админ-API; без него `/api/admin/*` отвечает `503` |
| `IMAGES_DIR` | нет | Каталог фото (по умолчанию `static/images`) |
| `RUST_LOG` | нет | Фильтр логирования |
| `OPENROUTER_API_KEY` | нет | Ключ для AI-генерации описаний; пусто — функция отключена |
| `OPENROUTER_MODEL` | нет | Модель OpenRouter (по умолчанию `openai/gpt-4o-mini`) |

Полный список и пояснения — в [`bec/README.md`](bec/README.md) и `bec/.env.example`.

---

## API (кратко)

Публичные маршруты только читают данные:

| Метод | Путь | Назначение |
|---|---|---|
| `GET` | `/` | Проверка сервера |
| `GET` | `/api/cars` | Автомобили в продаже (фильтр `status`, `limit`, `offset`) |
| `GET` | `/api/cars/latest` | Последнее добавленное авто |
| `GET` | `/api/cars/sold` | Проданные автомобили |
| `GET` | `/api/stats` | Статистика по статусам |
| `GET` | `/api/contacts` | Контакты автосалона |
| `GET` | `/ui/` | Публичная страница сайта |
| `GET` | `/partials/*` | HTML-фрагменты для htmx |
| `GET` | `/images/*` | Фотографии автомобилей |

Админ-маршруты (`/api/admin/*`) требуют заголовок `X-Admin-Token`:

| Метод | Путь | Назначение |
|---|---|---|
| `GET`/`POST` | `/api/admin/cars` | Список (включая проданные) / создание |
| `PATCH` | `/api/admin/cars/:id/status` | Смена статуса (в т.ч. «продано») |
| `DELETE` | `/api/admin/cars/:id` | Удаление авто и его фото |
| `POST`/`DELETE` | `/api/admin/cars/:id/images` | Загрузка / удаление фото |
| `GET` | `/api/admin/stats/visits` | Счётчик посещений сайта (только для админки) |

---

## Тесты

```bash
cd bec && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
# 33 passed

cd TG && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
# 8 passed

cd admin_app && flutter analyze && flutter test
```

Тесты бэкенда используют in-memory SQLite и не трогают рабочие файлы БД.

---

## Данные

В каталоге — только реальные автомобили:

- **Honda XR-V**, 2023, лот `0248`, 1 830 000 ₽;
- **Audi A3**, 2021, лот `0249`, 1 940 000 ₽.

Контактный телефон: `89996832070`. Адрес и остальные контакты не заполнены —
пустые блоки на сайте скрываются. Фейковых данных и демо-авто нет.

Исходные материалы (тексты, фото, описания) лежат в `real_data/`.

---

## Выкладка

Пошаговая инструкция по публикации на GitHub и развёртыванию на сервере —
в [`ТЗ_Миша_2026-10-20.md`](ТЗ_Миша_2026-10-20.md). Краткий план на день —
в [`Задание_Миша_2026-10-20.md`](Задание_Миша_2026-10-20.md).

Коротко:

1. Прогнать все тесты.
2. `cargo build --release` в `bec/`.
3. На сервере создать продовый `.env` (`APP_ENV=prod`, `ADMIN_TOKEN`, домены).
4. Запустить бинарник `target/release/axum_server` через systemd.
5. Поставить nginx-прокси на `127.0.0.1:8080` и HTTPS.
6. Проверить `/ui/`, `/api/*` и админку.

---

## Безопасность

- `.env`, токены, `*.db`, `target/`, `logs/` и `TG/data/` в репозиторий не попадают
  (см. `.gitignore`).
- Админ-API защищён `X-Admin-Token`; токен сравнивается за постоянное время.
- В проде Swagger отключён, наружу открыты только 80/443.
- Ошибки БД наружу не раскрываются — детали пишутся в логи.

---

## Документация

- [`bec/README.md`](bec/README.md) — API, конфигурация, миграции, логирование.
- [`admin_app/AGENTS.md`](admin_app/AGENTS.md) — архитектура и конвенции Flutter-приложения.
- [`TG/README.md`](TG/README.md) — правила именования файлов и работа бота.
- `bec/.env.example`, `TG/.env.example` — шаблоны конфигурации.
