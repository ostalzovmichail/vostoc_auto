# AGENTS.md

Flutter-приложение **«ВОСТОК АВТО · Управление»** — админ-клиент каталога автосалона.

> ⚠️ **Перед архитектурно значимыми изменениями** прочитайте
> `architecture/README.md`. Там описано, кто главный в разработке, что требует
> согласования и как подать предложение. Окончательные решения по агентам
> принимает человек.

## О проекте

- Управляет каталогом автомобилей через админ-API сайта: добавление, удаление,
  смена статуса (в т.ч. «Продано») и загрузка фотографий.
- Кодовая база одна для Android, iOS, Windows, Linux, macOS и Web.
- Данные не хранятся локально в БД приложения — только настройки подключения
  (shared_preferences). Все данные приходят с сервера.

## Технологии

| Слой | Что используется |
|---|---|
| Язык | Dart (SDK `^3.13.5`), Flutter stable |
| UI | Material 3, seed-цвет `0xFF0B5FFF`, светлая и тёмная темы |
| Состояние | **Riverpod 3** (`flutter_riverpod` + `riverpod_annotation` + code-gen) |
| Сеть | `http` (JSON + multipart-загрузка) |
| Хранение настроек | `shared_preferences` |
| Выбор файлов | `file_picker` (байты, а не путь — работает и на web) |
| Тесты | `flutter_test`, линтер `flutter_lints ^6.0.0` |

### Управление состоянием (Riverpod)

Проект **использует Riverpod 3**, а не `provider`/`ChangeNotifier`:

- Точка входа оборачивает приложение в `ProviderScope` (`main.dart`).
- Провайдеры объявляются через аннотацию `@riverpod` (`riverpod_annotation`)
  и генерируются `riverpod_generator` — не пишите провайдеры вручную.
- Экраны, которым нужен `ref`, наследуют `ConsumerWidget` /
  `ConsumerStatefulWidget`; доступ к значениям — `ref.watch` (реактивно) и
  `ref.read` (разово, внутри обработчиков).
- Функциональные провайдеры принимают `Ref ref` (в Riverpod 3 тип не
  параметризуется: `Ref`, а не `SomethingRef`).
- Сгенерированные файлы `*.g.dart` создаются `build_runner` и **не правятся
  руками**; при изменении провайдеров запускайте `dart run build_runner build`.
- Каждый файл с `@riverpod` объявляет `part '<name>.g.dart';`.
- Линт `riverpod_lint` (через `custom_lint`) обязателен — следуйте его
  подсказкам.

Ключевые провайдеры:

- `appSettingsProvider` (`services/settings.dart`) — настройки подключения,
  состояние `AppSettingsState` (`loading` / `loaded`).
- `apiClientProvider` (`services/api_client.dart`) — `ApiClient`, пересобирается
  при смене настроек через `ref.watch(appSettingsProvider)`.
- `carsListProvider`, `carStatusFilterProvider` (`providers/cars_providers.dart`) —
  каталог и фильтр статуса.

## Структура

```
lib/
├── main.dart                     # Точка входа, ProviderScope, темы, RootScreen
├── models/
│   ├── car.dart                  # Car (ответ API), CarDraft (запрос), CarStatus, форматирование
│   └── site_visits.dart          # SiteVisits (счётчик посещений сайта из админ-API)
├── providers/
│   └── cars_providers.dart       # carStatusFilterProvider, carsListProvider (+ .g.dart)
├── services/
│   ├── api_client.dart           # HTTP-клиент админ-API + apiClientProvider
│   ├── settings.dart             # AppSettings + appSettingsProvider (shared_preferences)
│   └── photo_picker.dart         # Выбор/чтение изображений в байты (кроссплатформенно)
├── utils/
│   ├── dialogs.dart              # confirmDialog, askTextDialog
│   └── ui_feedback.dart          # showMessage (SnackBar, единая точка)
├── widgets/                      # Общие переиспользуемые виджеты
│   ├── car_image.dart            # CarImage: сетевое изображение + плейсхолдер/ошибка
│   ├── message_view.dart         # MessageView: пустое/ошибочное состояние списка
│   └── status_chip.dart          # StatusChip + carStatusColor
└── screens/
    ├── home/
    │   ├── home_screen.dart      # Тонкий оркестратор каталога
    │   └── widgets/
    │       ├── car_card.dart     # Карточка авто в списке
    │       ├── status_filter_bar.dart  # Ряд чипов-фильтров
    │       └── visits_dialog.dart      # Диалог счётчика посещений сайта
    ├── car_detail/
    │   ├── car_detail_screen.dart      # Тонкий оркестратор карточки
    │   ├── photo_viewer_screen.dart    # Полноэкранный просмотр фото
    │   └── widgets/
    │       ├── car_cover.dart          # Обложка 16:9
    │       ├── price_card.dart         # Цена
    │       ├── specs_card.dart         # Характеристики + InfoRow
    │       └── photos_section.dart     # Секция фото + PhotoTile
    ├── car_form/
    │   ├── car_form_screen.dart        # Форма добавления
    │   └── widgets/
    │       └── car_form_field.dart     # Поле формы + валидаторы
    └── settings/
        └── settings_screen.dart        # Подключение: адрес, токен, «Проверить связь»
test/
├── car_model_test.dart           # Парсинг Car.fromJson, статусы, форматирование
└── site_visits_test.dart         # Парсинг SiteVisits.fromJson (счётчик посещений)
```

Сгенерированные `*.g.dart` (`services/api_client.g.dart`,
`services/settings.g.dart`, `providers/cars_providers.g.dart`) не редактируются
вручную.

## Команды

```bash
flutter pub get                          # установить зависимости
dart run build_runner build              # кодогенерация Riverpod (*.g.dart)
flutter run                              # запуск (иначе -d chrome / -d macos / -d android / -d ios)
flutter analyze                          # статический анализ
flutter test                             # тесты
flutter build apk                        # сборка Android (аналогично для ios/web/macos/windows/linux)
```

> Версии тулинга: `flutter_riverpod ^3.1.0`, `riverpod_annotation ^4.0.0`,
> `riverpod_generator ^4.0.0+1`, `riverpod_lint ^3.1.0`, `build_runner ^2.15.1`.
> Riverpod 2 не совместим с Dart SDK `^3.13.5` (устаревший `analyzer` ломает
> кодогенерацию) — не откатывайте версии вниз.

## Бэкенд и API

- Сервер — Rust-проект `../bec` (в этой же рабочей области `../`), по умолчанию
  `http://127.0.0.1:8080`. Все маршруты: `/api/admin/*`.
- Аутентификация: заголовок `X-Admin-Token` (значение `ADMIN_TOKEN` из `bec/.env` /
  `bec/.env.dev`). Токен — секрет: не коммитить `.env`, не публиковать.
- Ошибки сервера приходят в теле `{"message": "..."}` — их текст показывается
  пользователю в SnackBar (см. `ApiClient._decode`).
- Endpoint'ы: список/создание `GET|POST /api/admin/cars`, статус
  `PATCH /api/admin/cars/{id}/status`, удаление авто `DELETE /api/admin/cars/{id}`,
  фото `POST|DELETE /api/admin/cars/{id}/images`, счётчик посещений сайта
  `GET /api/admin/stats/visits` (на публичной странице не показывается).

## Важные договорённости

- **Статусы.** `enum CarStatus` (`stock / transit / order / sold`) держится строго
  синхронно с `CAR_STATUSES` в `bec/src/core/models.rs`. Новый статус на сервере
  → добавить в enum. Неизвестный статус из API считается `stock` (список не должен
  падать).
- **Ключи JSON** — snake_case (`lot_number`, `price_rub`, `created_at`), поля
  опциональные могут прийти `null` — парсинг не должен падать
  (`_asInt`/`_asString` с безопасными приведениями).
- **Photos на web.** Файлы читаются в байты (`file_picker` + `readAsBytes`), путь
  к файлу не используется — единый код для всех платформ.
- **Сетевой таймаут** — 20 секунд, чтобы «зависший» сервер не вешал UI.
- **Ошибки сети** — через `ApiException` с человекочитаемым сообщением;
  по всем операциям — SnackBar (`isError` → цвет `colorScheme.error`).
- **Форматирование.** `formatRub` — «1 940 000 ₽», `formatMileage` — «55 200 км»
  (разделитель тысяч — пробел).
- **Тема.** Только Material 3: не создавать виджеты в стиле Material 2
  (`FlatButton`, `RaisedButton` и т.п.).

## Чистая архитектура и чистый код

- **Разделение слоёв.** `models` — данные, `services` — доступ к API/хранилищу,
  `screens` — UI. UI не знает про `http`/`shared_preferences`, а сервисы не знают
  про `BuildContext`.
- **Единая ответственность.** Один класс/метод решает одну задачу. Экраны не
  содержат сетевой логики, модели не содержат логики представления.
- **DRY без фанатизма.** Повторяющуюся логику выносите в общие функции/виджеты
  (например, форматирование в `car.dart`, диалоги подтверждения, показ ошибок),
  но не создавайте абстракции «на будущее» и не жертвуйте читаемостью.
- **Именование** осмысленное и по конвенции: файлы/классы экранов —
  `*_screen.dart` / `*Screen`, приватные детали — `_private`.
- **Документирующие комментарии на русском** (`///`), ключевые решения — в теле
  кода. Комментируйте «почему», а не «что».
- **Без мёртвого кода** и закомментированных блоков — удаляйте, история в VCS.
- Строго следуем `flutter analyze` и `riverpod_lint`: правки проходят без новых
  warning'ов. `dart format` — обязателен.

## Производительность

- **Минимум перестроений.** В Riverpod используйте `ref.watch` только для того,
  что влияет на UI; разовые действия — через `ref.read` в обработчиках.
  Дробите провайдеры, чтобы изменение фильтра не перестраивало весь экран.
- **Ленивые списки.** Длинные списки — только `ListView.builder`/`separated`
  (никогда `ListView(children: [...])` для больших коллекций).
- **Сеть.** Не дублируйте запросы: кэшируйте через провайдеры, инвалидируйте
  точечно (`ref.invalidate(...)`) после изменений. Не создавайте `ApiClient`
  на каждый кадр — берите из `apiClientProvider`.
- **Изображения.** Задавайте размеры и `fit`, используйте `errorBuilder`;
  для тяжёлых сетевых картинок — кэширование (`cached_network_image`) при
  необходимости.
- **Ресурсы.** Контроллеры/подписки освобождайте в `dispose`, тяжёлые вычисления
  не выполняйте в `build`.

## Безопасность

- **Токен — секрет.** `X-Admin-Token` не логировать, не показывать в UI в
  открытом виде (используйте `obscureText`), не коммитить `.env` и токены.
- **Никаких секретов в коде и репозитории** — только через настройки/окружение.
- **Только HTTPS в проде.** `http://` допустим лишь для локальной разработки;
  не ослабляйте проверку сертификатов.
- **Валидация ввода.** Проверяйте данные формы до отправки (`Form` + валидаторы),
  не доверяйте ответам сервера: парсинг безопасный, без падений на `null`.
- **Минимум разрешений и данных.** Запрашивайте только нужные файлы/разрешения,
  не сохраняйте лишнее локально.
- **Ошибки — без утечек.** Пользователю показываем понятное сообщение, а не
  стектрейс/внутренние детали.

## Декомпозиция UI

- **Экран — тонкий оркестратор.** `*_screen.dart` держит состояние и навигацию,
  а разметку выносит в виджеты. Ориентир: экран ≤ ~250 строк; если больше —
  делите.
- **Виджеты рядом с экраном** — в `screens/<feature>/widgets/`. Общие для
  нескольких экранов — в `lib/widgets/`.
- **Переиспользуемое — выносим.** Повторяющиеся куски (изображение с
  плейсхолдером, чип статуса, пустое состояние, поля формы, диалоги, SnackBar)
  живут в `widgets/` и `utils/`.
- **Виджеты — по возможности `StatelessWidget`/`ConsumerWidget`** и без
  бизнес-логики: получают данные и колбэки через параметры.
- **Бизнес-логика — не в UI.** Сеть/парсинг — в `services`/`models`, состояние —
  в `providers`, вспомогательное — в `utils`.

## Конвенции кода

- **`ApiClient` не хранить глобально.** Он собирается из `AppSettings` через
  `apiClientProvider` и автоматически пересобирается при смене настроек.
- Обновление списка после действий: инвалидируйте `carsListProvider` после
  возврата из дочернего экрана или после операции (см. `_openDetail` / `_openForm`
  / `_run` в `home_screen.dart`).
- Провайдеры объявляются рядом со своей зоной ответственности; сгенерированные
  `*.g.dart` не редактируются.
- Названия файлов/классов Screen — `*_screen.dart` / `*Screen`.

## Платформенные нюансы

- На Android-эмуляторе хост доступен как `10.0.2.2`, не `127.0.0.1` —
  `AppSettings.defaultBaseUrl()` уже это учитывает.
- Web: нужен origin в `CORS_ALLOWED_ORIGINS` на сервере (`bec/.env`).
- Windows-сборка только на Windows; macOS/iOS — только на macOS.
