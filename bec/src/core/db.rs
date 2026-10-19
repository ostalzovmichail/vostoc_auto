use crate::core::models::{Car, CarFilter, Contact, NewCar, Stats, StatusUpdate, VisitStats};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::time::Duration;

/// Максимальное количество соединений в пуле.
const MAX_CONNECTIONS: u32 = 5;
/// Сколько ждать свободное соединение, прежде чем вернуть ошибку.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(3);

/// Трейт операций с базой данных.
///
/// Обработчики depend от трейта, а не от конкретной реализации: чтобы
/// перейти с SQLite на PostgreSQL/MySQL, достаточно написать новую
/// реализацию трейта и не трогать остальной код (принцип Dependency Inversion).
#[async_trait::async_trait]
pub trait Database: Send + Sync {
    /// Возвращает версию SQLite — полезно как быстрая проверка живости БД.
    async fn get_version(&self) -> Result<String, sqlx::Error>;

    /// Последнее добавленное авто — акцент на главной странице.
    async fn latest_car(&self) -> Result<Option<Car>, sqlx::Error>;

    /// Список автомобилей. `status = None` — все, кроме проданных;
    /// иначе только записи с указанным статусом (`stock` | `transit` | `order`).
    async fn list_cars(
        &self,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Car>, sqlx::Error>;

    /// Проданные автомобили, свежие сделки — первыми.
    async fn sold_cars(&self, limit: i64) -> Result<Vec<Car>, sqlx::Error>;

    /// Каталог с произвольным набором фильтров — для страниц на htmx.
    async fn filter_cars(&self, filter: &CarFilter) -> Result<Vec<Car>, sqlx::Error>;

    /// Средний срок сделки в днях по проданным авто (если данные есть).
    async fn avg_deal_duration_days(&self) -> Result<Option<i64>, sqlx::Error>;

    /// Сводная статистика по базе.
    async fn cars_stats(&self) -> Result<Stats, sqlx::Error>;

    /// Контактная информация автосалона.
    async fn contacts(&self) -> Result<Option<Contact>, sqlx::Error>;

    /// Учитывает просмотр публичной страницы сайта (дневной агрегат).
    async fn record_site_visit(&self) -> Result<(), sqlx::Error>;

    /// Счётчик посещений сайта для админки.
    async fn site_visits(&self) -> Result<VisitStats, sqlx::Error>;

    // ── Админ-API: управление каталогом ────────────────────────────────────

    /// Все автомобили, включая проданные. `status = None` — без фильтра.
    async fn admin_list_cars(
        &self,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Car>, sqlx::Error>;

    /// Один автомобиль по внутреннему `id`.
    async fn car_by_id(&self, id: i64) -> Result<Option<Car>, sqlx::Error>;

    /// Следующий свободный номер лота (например, `0250`).
    async fn next_lot_number(&self) -> Result<String, sqlx::Error>;

    /// Создаёт автомобиль и возвращает его с присвоенным `id`.
    async fn create_car(&self, new: &NewCar) -> Result<Car, sqlx::Error>;

    /// Меняет статус. При переводе в `sold` проставляет дату продажи и срок
    /// сделки; при возврате из `sold` — очищает их. `None`, если авто нет.
    async fn set_car_status(
        &self,
        id: i64,
        update: &StatusUpdate,
    ) -> Result<Option<Car>, sqlx::Error>;

    /// Удаляет автомобиль вместе с галереей. Возвращает URL-ы удалённых фото
    /// (чтобы вызывающий мог убрать файлы с диска); `None`, если авто нет.
    async fn delete_car(&self, id: i64) -> Result<Option<Vec<String>>, sqlx::Error>;

    /// Добавляет фото в галерею, возвращает занятую позицию.
    async fn add_car_image(&self, car_id: i64, url: &str) -> Result<i64, sqlx::Error>;

    /// Удаляет фото по URL. Возвращает `false`, если такого фото не было.
    async fn delete_car_image(&self, car_id: i64, url: &str) -> Result<bool, sqlx::Error>;
}

/// Реализация [`Database`] поверх пула соединений SQLite.
pub struct SqliteDb {
    pool: SqlitePool,
}

impl SqliteDb {
    /// Подключается к БД по `database_url` и прогоняет миграции.
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(MAX_CONNECTIONS)
            .acquire_timeout(ACQUIRE_TIMEOUT)
            .connect(database_url)
            .await?;

        // Миграции встроены в бинарь макросом `migrate!` (папка ./migrations)
        // и выполняются при старте. Это исключает подмену SQL-файлов на диске.
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    /// Догружает галерею изображений в уже выбранные авто.
    ///
    /// Отдельный запрос на каждое авто, а не JOIN со склейкой строк: фотографий
    /// мало, зато код не зависит от диалекта SQL и не требует ручного разбора
    /// склеенного списка. При заметном росте данных заменим на один запрос
    /// `WHERE car_id IN (...)`.
    async fn attach_images(&self, cars: &mut [Car]) -> Result<(), sqlx::Error> {
        for car in cars.iter_mut() {
            let rows: Vec<(String,)> =
                sqlx::query_as("SELECT url FROM car_images WHERE car_id = ? ORDER BY position")
                    .bind(car.id)
                    .fetch_all(&self.pool)
                    .await?;
            car.images = rows.into_iter().map(|(url,)| url).collect();
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl Database for SqliteDb {
    async fn get_version(&self) -> Result<String, sqlx::Error> {
        // Запрос статический и без подстановок — SQL-инъекция исключена.
        // Если появятся параметры, используем `sqlx::query_as(...).bind(...)`,
        // а не форматирование строки.
        let row: (String,) = sqlx::query_as("SELECT sqlite_version()")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    async fn latest_car(&self) -> Result<Option<Car>, sqlx::Error> {
        // Сортировка по дате и id: id — надёжный тай-брейкер, если два авто
        // добавлены в одну секунду (по умолчанию CURRENT_TIMESTAMP неточен).
        let mut car = sqlx::query_as::<_, Car>(
            "SELECT * FROM cars ORDER BY created_at DESC, id DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(car) = car.as_mut() {
            self.attach_images(std::slice::from_mut(car)).await?;
        }

        Ok(car)
    }

    async fn list_cars(
        &self,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Car>, sqlx::Error> {
        // Два варианта запроса вместо одного с `(? IS NULL OR status = ?)`:
        // так условие по статусу остаётся статическим и попадает в план
        // запроса, а индекс idx_cars_status действительно используется.
        let mut cars = match status {
            Some(status) => {
                sqlx::query_as::<_, Car>(
                    "SELECT * FROM cars WHERE status = ? \
                     ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?",
                )
                .bind(status)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, Car>(
                    "SELECT * FROM cars WHERE status != 'sold' \
                     ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?",
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        }?;

        self.attach_images(&mut cars).await?;
        Ok(cars)
    }

    async fn sold_cars(&self, limit: i64) -> Result<Vec<Car>, sqlx::Error> {
        // COALESCE: если sold_at вдруг пуст, сортируем по дате добавления —
        // запись не «улетит» в конец списка с NULL.
        let mut cars = sqlx::query_as::<_, Car>(
            "SELECT * FROM cars WHERE status = 'sold' \
             ORDER BY COALESCE(sold_at, created_at) DESC, id DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        self.attach_images(&mut cars).await?;
        Ok(cars)
    }

    async fn filter_cars(&self, filter: &CarFilter) -> Result<Vec<Car>, sqlx::Error> {
        // Условия собираем через QueryBuilder: их набор произвольный, но
        // значения всегда уходят отдельными bind-параметрами — конкатенация
        // пользовательского ввода в SQL исключена.
        let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT * FROM cars WHERE 1 = 1");

        match filter.status.as_deref().filter(|value| !value.is_empty()) {
            Some(status) => {
                query.push(" AND status = ").push_bind(status.to_string());
            }
            None => {
                // Без явного статуса проданные не показываем — как в /api/cars.
                query.push(" AND status != 'sold'");
            }
        }

        if let Some(brand) = filter
            .brand
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            query
                .push(" AND lower(brand) = ")
                .push_bind(brand.to_lowercase());
        }

        if let Some(needle) = filter
            .query
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let like = format!("%{}%", needle.to_lowercase());
            query
                .push(" AND (lower(brand) LIKE ")
                .push_bind(like.clone())
                .push(" OR lower(model) LIKE ")
                .push_bind(like.clone())
                .push(" OR lower(lot_number) LIKE ")
                .push_bind(like)
                .push(")");
        }

        // Значение сортировки не подставляется в SQL: выбираем из белого списка.
        let order = match filter.sort.as_deref() {
            Some("price-asc") => " ORDER BY price_rub ASC",
            Some("price-desc") => " ORDER BY price_rub DESC",
            Some("mileage") => " ORDER BY mileage ASC, id DESC",
            _ => " ORDER BY created_at DESC, id DESC",
        };
        query.push(order);
        query
            .push(" LIMIT ")
            .push_bind(filter.limit)
            .push(" OFFSET ")
            .push_bind(filter.offset);

        let mut cars = query.build_query_as::<Car>().fetch_all(&self.pool).await?;
        self.attach_images(&mut cars).await?;
        Ok(cars)
    }

    async fn avg_deal_duration_days(&self) -> Result<Option<i64>, sqlx::Error> {
        // Агрегат всегда возвращает одну строку; при отсутствии данных AVG даёт
        // NULL, а CAST сохраняет NULL — отсюда Option.
        sqlx::query_scalar(
            "SELECT CAST(AVG(deal_duration_days) AS INTEGER) \
             FROM cars WHERE deal_duration_days IS NOT NULL",
        )
        .fetch_one(&self.pool)
        .await
    }

    async fn cars_stats(&self) -> Result<Stats, sqlx::Error> {
        // COUNT(CASE ...) вместо SUM(status = '...'): COUNT никогда не
        // возвращает NULL, поэтому тип результата всегда i64.
        sqlx::query_as::<_, Stats>(
            "SELECT \
                COUNT(*) AS total, \
                COUNT(CASE WHEN status = 'stock'   THEN 1 END) AS in_stock, \
                COUNT(CASE WHEN status = 'transit' THEN 1 END) AS in_transit, \
                COUNT(CASE WHEN status = 'order'   THEN 1 END) AS on_order, \
                COUNT(CASE WHEN status = 'sold'    THEN 1 END) AS sold \
             FROM cars",
        )
        .fetch_one(&self.pool)
        .await
    }

    async fn contacts(&self) -> Result<Option<Contact>, sqlx::Error> {
        sqlx::query_as::<_, Contact>("SELECT * FROM contacts WHERE id = 1")
            .fetch_optional(&self.pool)
            .await
    }

    async fn record_site_visit(&self) -> Result<(), sqlx::Error> {
        // Один UPSERT вместо «прочитать → прибавить → записать»: при
        // параллельных запросах не теряются просмотры. День — по VLAT.
        sqlx::query(
            "INSERT INTO site_visits (day, visits) \
             VALUES (date('now', '+10 hours'), 1) \
             ON CONFLICT(day) DO UPDATE SET visits = visits + 1",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn site_visits(&self) -> Result<VisitStats, sqlx::Error> {
        sqlx::query_as::<_, VisitStats>(
            "SELECT \
                COALESCE(SUM(visits), 0) AS total, \
                COALESCE(MAX(CASE WHEN day = date('now', '+10 hours') \
                                  THEN visits END), 0) AS today, \
                COALESCE(SUM(CASE WHEN day >= date('now', '+10 hours', '-6 days') \
                                  THEN visits ELSE 0 END), 0) AS last_7_days \
             FROM site_visits",
        )
        .fetch_one(&self.pool)
        .await
    }

    async fn admin_list_cars(
        &self,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Car>, sqlx::Error> {
        // Отличие от `list_cars`: без фильтра отдаём ВСЁ, включая проданные —
        // админке нужен полный список для управления.
        let mut cars = match status {
            Some(status) => {
                sqlx::query_as::<_, Car>(
                    "SELECT * FROM cars WHERE status = ? \
                     ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?",
                )
                .bind(status)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, Car>(
                    "SELECT * FROM cars ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?",
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        }?;

        self.attach_images(&mut cars).await?;
        Ok(cars)
    }

    async fn car_by_id(&self, id: i64) -> Result<Option<Car>, sqlx::Error> {
        let mut car = sqlx::query_as::<_, Car>("SELECT * FROM cars WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(car) = car.as_mut() {
            self.attach_images(std::slice::from_mut(car)).await?;
        }

        Ok(car)
    }

    async fn next_lot_number(&self) -> Result<String, sqlx::Error> {
        // Номера лотов — числовые строки с ведущими нулями ("0249"), поэтому
        // сравниваем их как числа (CAST), а не как текст: иначе "0249" < "1000".
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT lot_number FROM cars ORDER BY CAST(lot_number AS INTEGER) DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        let next = row
            .and_then(|(lot,)| lot.trim().parse::<i64>().ok())
            .map(|value| value + 1)
            .unwrap_or(1);

        Ok(format!("{next:04}"))
    }

    async fn create_car(&self, new: &NewCar) -> Result<Car, sqlx::Error> {
        let lot_number = match new
            .lot_number
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(lot) => lot.to_string(),
            None => self.next_lot_number().await?,
        };
        let status = new.status.clone().unwrap_or_else(|| "stock".to_string());

        let result = sqlx::query(
            "INSERT INTO cars (lot_number, brand, model, year, mileage, engine, \
             transmission, drive, price_rub, auction_price_yen, auction_name, \
             auction_lot, rating, status, transit_days, delivery_city, image_url, \
             created_at, sold_at, deal_duration_days) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, \
             datetime('now'), NULL, NULL)",
        )
        .bind(&lot_number)
        .bind(&new.brand)
        .bind(&new.model)
        .bind(new.year)
        .bind(new.mileage)
        .bind(&new.engine)
        .bind(&new.transmission)
        .bind(&new.drive)
        .bind(new.price_rub)
        .bind(new.auction_price_yen)
        .bind(&new.auction_name)
        .bind(&new.auction_lot)
        .bind(&new.rating)
        .bind(&status)
        .bind(new.transit_days)
        .bind(&new.delivery_city)
        .bind(&new.image_url)
        .execute(&self.pool)
        .await?;

        self.car_by_id(result.last_insert_rowid())
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn set_car_status(
        &self,
        id: i64,
        update: &StatusUpdate,
    ) -> Result<Option<Car>, sqlx::Error> {
        // Сначала убеждаемся, что авто есть: так обработчик честно отдаёт 404,
        // а не «успех» по несуществующей записи.
        if self.car_by_id(id).await?.is_none() {
            return Ok(None);
        }

        if update.status == "sold" {
            // deal_duration_days считаем из даты поступления; julianday даёт
            // разницу в сутках, CAST — целые дни.
            sqlx::query(
                "UPDATE cars SET status = 'sold', \
                 sold_at = datetime('now'), \
                 deal_duration_days = CAST(julianday('now') - julianday(created_at) AS INTEGER), \
                 delivery_city = COALESCE(?, delivery_city), \
                 transit_days = NULL \
                 WHERE id = ?",
            )
            .bind(&update.delivery_city)
            .bind(id)
            .execute(&self.pool)
            .await?;
        } else {
            // Возврат из «продано»: снимаем признаки сделки.
            sqlx::query(
                "UPDATE cars SET status = ?, transit_days = ?, \
                 delivery_city = NULL, sold_at = NULL, deal_duration_days = NULL \
                 WHERE id = ?",
            )
            .bind(&update.status)
            .bind(update.transit_days)
            .bind(id)
            .execute(&self.pool)
            .await?;
        }

        self.car_by_id(id).await
    }

    async fn delete_car(&self, id: i64) -> Result<Option<Vec<String>>, sqlx::Error> {
        // Галерею удаляем вручную, а не полагаемся на ON DELETE CASCADE:
        // у SQLite внешние ключи по умолчанию выключены, и «тихие» осиротевшие
        // строки в car_images нам не нужны.
        let mut tx = self.pool.begin().await?;

        let urls: Vec<(String,)> = sqlx::query_as("SELECT url FROM car_images WHERE car_id = ?")
            .bind(id)
            .fetch_all(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM car_images WHERE car_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        let deleted = sqlx::query("DELETE FROM cars WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        if deleted.rows_affected() == 0 {
            tx.rollback().await?;
            return Ok(None);
        }

        tx.commit().await?;
        Ok(Some(urls.into_iter().map(|(url,)| url).collect()))
    }

    async fn add_car_image(&self, car_id: i64, url: &str) -> Result<i64, sqlx::Error> {
        // Позиция = хвост галереи. COALESCE нужен для пустой галереи: MAX() = NULL.
        let (position,): (i64,) = sqlx::query_as(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM car_images WHERE car_id = ?",
        )
        .bind(car_id)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query("INSERT INTO car_images (car_id, position, url) VALUES (?, ?, ?)")
            .bind(car_id)
            .bind(position)
            .bind(url)
            .execute(&self.pool)
            .await?;

        // Если обложки ещё нет — назначаем ею первое загруженное фото,
        // чтобы карточка авто не осталась «пустой».
        sqlx::query(
            "UPDATE cars SET image_url = ? \
             WHERE id = ? AND (image_url IS NULL OR trim(image_url) = '')",
        )
        .bind(url)
        .bind(car_id)
        .execute(&self.pool)
        .await?;

        Ok(position)
    }

    async fn delete_car_image(&self, car_id: i64, url: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM car_images WHERE car_id = ? AND url = ?")
            .bind(car_id)
            .bind(url)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Ok(false);
        }

        // Обложку могло удалить — переназначаем на первое оставшееся фото
        // (подзапрос вернёт NULL, если галерея опустела).
        sqlx::query(
            "UPDATE cars SET image_url = \
             (SELECT url FROM car_images WHERE car_id = ? ORDER BY position LIMIT 1) \
             WHERE id = ?",
        )
        .bind(car_id)
        .bind(car_id)
        .execute(&self.pool)
        .await?;

        Ok(true)
    }
}
