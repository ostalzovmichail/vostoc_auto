//! Разбор и валидация HTML-форм админки.
//!
//! Браузер присылает все поля строками, а пустое необязательное поле — пустой
//! строкой. Поэтому форма сначала принимает «сырые» строки, затем собирает
//! [`NewCar`]/[`CarUpdate`]. Ошибки возвращаются списком, чтобы форма
//! перерисовалась с сообщениями, а не отдавала JSON или 500.

use serde::Deserialize;

use crate::core::models::{Car, CarUpdate, NewCar, StatusUpdate, CAR_STATUSES};

/// Сообщение об ошибке конкретного поля формы.
#[derive(Debug, Clone)]
pub struct FieldError {
    pub field: &'static str,
    pub message: String,
}

impl FieldError {
    fn new(field: &'static str, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }
}

/// Сырые значения формы автомобиля.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CarForm {
    #[serde(default)]
    pub lot_number: String,
    #[serde(default)]
    pub brand: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub year: String,
    #[serde(default)]
    pub mileage: String,
    #[serde(default)]
    pub engine: String,
    #[serde(default)]
    pub transmission: String,
    #[serde(default)]
    pub drive: String,
    #[serde(default)]
    pub price_rub: String,
    #[serde(default)]
    pub auction_price_yen: String,
    #[serde(default)]
    pub auction_name: String,
    #[serde(default)]
    pub auction_lot: String,
    #[serde(default)]
    pub rating: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub transit_days: String,
    #[serde(default)]
    pub delivery_city: String,
}

impl CarForm {
    /// Заполняет форму из существующего авто (страница редактирования).
    pub fn from_car(car: &Car) -> Self {
        Self {
            lot_number: car.lot_number.clone(),
            brand: car.brand.clone(),
            model: car.model.clone(),
            year: car.year.to_string(),
            mileage: car.mileage.to_string(),
            engine: car.engine.clone(),
            transmission: car.transmission.clone(),
            drive: car.drive.clone(),
            price_rub: car.price_rub.to_string(),
            auction_price_yen: car
                .auction_price_yen
                .map(|value| value.to_string())
                .unwrap_or_default(),
            auction_name: car.auction_name.clone().unwrap_or_default(),
            auction_lot: car.auction_lot.clone().unwrap_or_default(),
            rating: car.rating.clone().unwrap_or_default(),
            status: car.status.clone(),
            transit_days: car
                .transit_days
                .map(|value| value.to_string())
                .unwrap_or_default(),
            delivery_city: car.delivery_city.clone().unwrap_or_default(),
        }
    }

    /// Значение по умолчанию для новой записи.
    pub fn with_defaults() -> Self {
        Self {
            status: "stock".to_string(),
            ..Self::default()
        }
    }

    /// Собирает данные для создания автомобиля.
    pub fn to_new_car(&self) -> Result<NewCar, Vec<FieldError>> {
        let mut errors = Vec::new();

        let brand = self.required("brand", "марку", &mut errors);
        let model = self.required("model", "модель", &mut errors);
        let year = self.int("year", "Год выпуска", 1950, 2100, &mut errors);
        let mileage = self.int("mileage", "Пробег", 0, i64::MAX, &mut errors);
        let engine = self.required("engine", "двигатель", &mut errors);
        let transmission = self.required("transmission", "коробку передач", &mut errors);
        let drive = self.required("drive", "привод", &mut errors);
        let price_rub = self.int("price_rub", "Цену", 1, i64::MAX, &mut errors);
        let auction_price_yen =
            self.optional_int("auction_price_yen", "Цену аукциона", 0, &mut errors);
        let transit_days = self.optional_int("transit_days", "Срок доставки", 0, &mut errors);

        let status = if self.status.trim().is_empty() {
            "stock".to_string()
        } else {
            self.status.trim().to_string()
        };
        if !CAR_STATUSES.contains(&status.as_str()) {
            errors.push(FieldError::new("status", "Недопустимый статус"));
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(NewCar {
            lot_number: self.optional_text("lot_number"),
            brand,
            model,
            year,
            mileage,
            engine,
            transmission,
            drive,
            price_rub,
            auction_price_yen,
            auction_name: self.optional_text("auction_name"),
            auction_lot: self.optional_text("auction_lot"),
            rating: self.optional_text("rating"),
            status: Some(status),
            transit_days,
            delivery_city: self.optional_text("delivery_city"),
            image_url: None,
        })
    }

    /// Собирает данные для редактирования характеристик.
    pub fn to_car_update(&self) -> Result<CarUpdate, Vec<FieldError>> {
        let mut errors = Vec::new();

        let lot_number = self.required("lot_number", "номер лота", &mut errors);
        let brand = self.required("brand", "марку", &mut errors);
        let model = self.required("model", "модель", &mut errors);
        let year = self.int("year", "Год выпуска", 1950, 2100, &mut errors);
        let mileage = self.int("mileage", "Пробег", 0, i64::MAX, &mut errors);
        let engine = self.required("engine", "двигатель", &mut errors);
        let transmission = self.required("transmission", "коробку передач", &mut errors);
        let drive = self.required("drive", "привод", &mut errors);
        let price_rub = self.int("price_rub", "Цену", 1, i64::MAX, &mut errors);
        let auction_price_yen =
            self.optional_int("auction_price_yen", "Цену аукциона", 0, &mut errors);

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(CarUpdate {
            lot_number,
            brand,
            model,
            year,
            mileage,
            engine,
            transmission,
            drive,
            price_rub,
            auction_price_yen,
            auction_name: self.optional_text("auction_name"),
            auction_lot: self.optional_text("auction_lot"),
            rating: self.optional_text("rating"),
        })
    }

    fn required(&self, field: &'static str, label: &str, errors: &mut Vec<FieldError>) -> String {
        let value = self.value(field).trim().to_string();
        if value.is_empty() {
            errors.push(FieldError::new(field, format!("Укажите {label}")));
        }
        value
    }

    fn int(
        &self,
        field: &'static str,
        label: &str,
        min: i64,
        max: i64,
        errors: &mut Vec<FieldError>,
    ) -> i64 {
        let raw = self.value(field).trim().to_string();
        if raw.is_empty() {
            errors.push(FieldError::new(field, format!("{label}: поле обязательно")));
            return 0;
        }
        match raw.replace(' ', "").parse::<i64>() {
            Ok(value) if (min..=max).contains(&value) => value,
            Ok(_) => {
                errors.push(FieldError::new(
                    field,
                    format!("{label}: допустимое значение — от {min} до {max}"),
                ));
                0
            }
            Err(_) => {
                errors.push(FieldError::new(field, format!("{label}: ожидается число")));
                0
            }
        }
    }

    fn optional_int(
        &self,
        field: &'static str,
        label: &str,
        min: i64,
        errors: &mut Vec<FieldError>,
    ) -> Option<i64> {
        let raw = self.value(field).trim();
        if raw.is_empty() {
            return None;
        }
        match raw.replace(' ', "").parse::<i64>() {
            Ok(value) if value >= min => Some(value),
            _ => {
                errors.push(FieldError::new(
                    field,
                    format!("{label}: ожидается число не меньше {min}"),
                ));
                None
            }
        }
    }

    fn optional_text(&self, field: &'static str) -> Option<String> {
        let value = self.value(field).trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }

    fn value(&self, field: &'static str) -> &str {
        match field {
            "lot_number" => &self.lot_number,
            "brand" => &self.brand,
            "model" => &self.model,
            "year" => &self.year,
            "mileage" => &self.mileage,
            "engine" => &self.engine,
            "transmission" => &self.transmission,
            "drive" => &self.drive,
            "price_rub" => &self.price_rub,
            "auction_price_yen" => &self.auction_price_yen,
            "auction_name" => &self.auction_name,
            "auction_lot" => &self.auction_lot,
            "rating" => &self.rating,
            "status" => &self.status,
            "transit_days" => &self.transit_days,
            "delivery_city" => &self.delivery_city,
            _ => "",
        }
    }
}

/// Форма входа.
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    #[serde(default)]
    pub token: String,
}

/// Форма смены статуса.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct StatusForm {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub transit_days: String,
    #[serde(default)]
    pub delivery_city: String,
}

impl StatusForm {
    pub fn to_update(&self) -> Result<StatusUpdate, String> {
        let status = self.status.trim();
        if !CAR_STATUSES.contains(&status) {
            return Err("Недопустимый статус".to_string());
        }

        let transit_days = if self.transit_days.trim().is_empty() {
            None
        } else {
            match self.transit_days.trim().replace(' ', "").parse::<i64>() {
                Ok(value) if value >= 0 => Some(value),
                _ => return Err("Срок доставки: ожидается неотрицательное число".to_string()),
            }
        };

        let delivery_city = if self.delivery_city.trim().is_empty() {
            None
        } else {
            Some(self.delivery_city.trim().to_string())
        };

        Ok(StatusUpdate {
            status: status.to_string(),
            transit_days,
            delivery_city,
        })
    }
}

/// Параметры удаления/перестановки фото (URL приходит скрытым полем).
#[derive(Debug, Deserialize)]
pub struct ImageForm {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub direction: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_form() -> CarForm {
        CarForm {
            brand: "Honda".to_string(),
            model: "Fit".to_string(),
            year: "2022".to_string(),
            mileage: "30 000".to_string(),
            engine: "1.5".to_string(),
            transmission: "AT".to_string(),
            drive: "FWD".to_string(),
            price_rub: "1500000".to_string(),
            ..CarForm::with_defaults()
        }
    }

    #[test]
    fn valid_form_creates_car() {
        let car = valid_form()
            .to_new_car()
            .expect("форма должна быть валидной");
        assert_eq!(car.brand, "Honda");
        assert_eq!(car.mileage, 30_000);
        assert_eq!(car.status.as_deref(), Some("stock"));
    }

    #[test]
    fn empty_required_fields_are_errors() {
        let errors = CarForm::default()
            .to_new_car()
            .expect_err("пустая форма невалидна");
        assert!(errors.iter().any(|error| error.field == "brand"));
        assert!(errors.iter().any(|error| error.field == "price_rub"));
    }

    #[test]
    fn invalid_year_is_an_error() {
        let mut form = valid_form();
        form.year = "1800".to_string();
        let errors = form.to_new_car().expect_err("год вне диапазона");
        assert!(errors.iter().any(|error| error.field == "year"));
    }

    #[test]
    fn empty_lot_is_allowed_on_create_but_not_on_update() {
        let form = valid_form();
        assert!(form.to_new_car().is_ok());
        assert!(form.to_car_update().is_err());
    }

    #[test]
    fn status_form_validates_status() {
        let mut form = StatusForm {
            status: "sold".to_string(),
            delivery_city: "Москва".to_string(),
            ..StatusForm::default()
        };
        let update = form.to_update().expect("sold допустим");
        assert_eq!(update.delivery_city.as_deref(), Some("Москва"));

        form.status = "hacked".to_string();
        assert!(form.to_update().is_err());
    }
}
