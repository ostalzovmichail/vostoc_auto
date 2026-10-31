//! Изменяющие действия веб-админки.
//!
//! Все маршруты принимают обычную HTML-форму (`application/x-www-form-urlencoded`
//! или multipart), поэтому работают и с htmx, и без JavaScript. При htmx-запросе
//! возвращается фрагмент; при обычном POST — редирект.

use askama::Template;
use axum::body::Body;
use axum::extract::{Multipart, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Form, Json};

use crate::api::admin::auth::is_htmx;
use crate::api::admin::forms::{CarForm, FieldError, ImageForm, StatusForm};
use crate::api::admin::media::{
    read_images, remove_empty_dir, remove_image_file, remove_saved, save_image, IMAGES_PREFIX,
};
use crate::api::admin::pages::error_page;
use crate::api::admin::templates::{
    AiDescriptionTemplate, CarFormFragmentTemplate, CarFormPageTemplate, PhotosGridTemplate,
    StatusCardTemplate, ToastTemplate,
};
use crate::api::handlers::ai::{describe_car, DescribeCarRequest};
use crate::api::AppState;
use crate::core::models::Car;

/// Заголовок htmx для клиентского редиректа.
const HX_REDIRECT: &str = "hx-redirect";

/// POST /admin/cars — создать автомобиль.
pub async fn create_car(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<CarForm>,
) -> Response {
    let new = match form.to_new_car() {
        Ok(new) => new,
        Err(errors) => {
            return form_errors(
                &form,
                errors,
                false,
                "/admin/cars".to_string(),
                None,
                &headers,
            )
        }
    };

    match state.db.create_car(&new).await {
        Ok(car) => {
            tracing::info!(id = car.id, lot = %car.lot_number, "Автомобиль добавлен (веб-админка)");
            redirect(&headers, &format!("/admin/cars/{}", car.id))
        }
        Err(error) => {
            tracing::error!(error = %error, "Не удалось создать автомобиль (веб-админка)");
            form_errors(
                &form,
                vec![FieldError {
                    field: "lot_number",
                    message: "Не удалось создать запись. Возможно, номер лота уже занят."
                        .to_string(),
                }],
                false,
                "/admin/cars".to_string(),
                None,
                &headers,
            )
        }
    }
}

/// POST /admin/cars/:id — изменить характеристики.
pub async fn update_car(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Form(form): Form<CarForm>,
) -> Response {
    let update = match form.to_car_update() {
        Ok(update) => update,
        Err(errors) => {
            return form_errors(
                &form,
                errors,
                true,
                format!("/admin/cars/{id}"),
                Some(id),
                &headers,
            )
        }
    };

    match state.db.update_car(id, &update).await {
        Ok(Some(car)) => {
            tracing::info!(id, lot = %car.lot_number, "Автомобиль изменён (веб-админка)");
            redirect(&headers, &format!("/admin/cars/{id}"))
        }
        Ok(None) => error_page(
            StatusCode::NOT_FOUND,
            "Автомобиль не найден",
            "Такой записи нет в каталоге.",
        ),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось изменить автомобиль (веб-админка)");
            form_errors(
                &form,
                vec![FieldError {
                    field: "lot_number",
                    message: "Не удалось сохранить изменения. Проверьте номер лота.".to_string(),
                }],
                true,
                format!("/admin/cars/{id}"),
                Some(id),
                &headers,
            )
        }
    }
}

/// POST /admin/cars/:id/status — сменить статус.
pub async fn set_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Form(form): Form<StatusForm>,
) -> Response {
    let update = match form.to_update() {
        Ok(update) => update,
        Err(message) => return status_error(&state, id, &headers, &message).await,
    };

    match state.db.set_car_status(id, &update).await {
        Ok(Some(car)) => {
            tracing::info!(id, status = %update.status, "Статус изменён (веб-админка)");
            if is_htmx(&headers) {
                let main = render_html(&StatusCardTemplate { car });
                html_response(with_toast(main, "ok", "Статус сохранён"))
            } else {
                redirect(&headers, &format!("/admin/cars/{id}"))
            }
        }
        Ok(None) => error_page(
            StatusCode::NOT_FOUND,
            "Автомобиль не найден",
            "Такой записи нет в каталоге.",
        ),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось изменить статус (веб-админка)");
            status_error(&state, id, &headers, "Не удалось сохранить статус").await
        }
    }
}

/// POST /admin/cars/:id/delete — удалить автомобиль и его фото.
pub async fn delete_car(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    match state.db.delete_car(id).await {
        Ok(Some(urls)) => {
            for url in &urls {
                remove_image_file(&state, url).await;
            }
            remove_empty_dir(&state, &urls).await;
            tracing::info!(id, photos = urls.len(), "Автомобиль удалён (веб-админка)");
            redirect(&headers, "/admin/cars")
        }
        Ok(None) => error_page(
            StatusCode::NOT_FOUND,
            "Автомобиль не найден",
            "Такой записи нет в каталоге.",
        ),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось удалить автомобиль (веб-админка)");
            error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Ошибка удаления",
                "Не удалось удалить автомобиль.",
            )
        }
    }
}

/// POST /admin/cars/:id/images — загрузить одно или несколько фото.
pub async fn upload_images(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    let Some(car) = fetch_car(&state, id).await else {
        return error_page(
            StatusCode::NOT_FOUND,
            "Автомобиль не найден",
            "Такой записи нет в каталоге.",
        );
    };

    let images = match read_images(multipart).await {
        Ok(images) => images,
        Err(message) => return photos_error(&state, id, &headers, &message).await,
    };

    let mut added = 0usize;
    let mut last_error: Option<String> = None;

    for (extension, bytes) in &images {
        match save_image(&state, &car, extension, bytes).await {
            Ok(saved) => match state.db.add_car_image(id, &saved.url).await {
                Ok(_) => added += 1,
                Err(error) => {
                    tracing::error!(error = %error, "Не удалось сохранить фото в БД");
                    remove_saved(&saved).await;
                    last_error = Some("Не удалось сохранить фото в базе".to_string());
                }
            },
            Err(message) => last_error = Some(message),
        }
    }

    if added == 0 {
        return photos_error(
            &state,
            id,
            &headers,
            last_error.as_deref().unwrap_or("Не удалось загрузить фото"),
        )
        .await;
    }

    tracing::info!(id, count = added, "Фотографии загружены (веб-админка)");
    photos_success(
        &state,
        id,
        &headers,
        if added == 1 {
            "Фото добавлено".to_string()
        } else {
            format!("Добавлено фото: {added}")
        },
    )
    .await
}

/// POST /admin/cars/:id/images/delete — удалить одно фото.
pub async fn delete_image(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Form(form): Form<ImageForm>,
) -> Response {
    let Some(url) = normalize_url(&form.url) else {
        return photos_error(&state, id, &headers, "Некорректный путь фото").await;
    };

    match state.db.delete_car_image(id, &url).await {
        Ok(true) => {
            remove_image_file(&state, &url).await;
            remove_empty_dir(&state, std::slice::from_ref(&url)).await;
            tracing::info!(id, url = %url, "Фото удалено (веб-админка)");
            photos_success(&state, id, &headers, "Фото удалено".to_string()).await
        }
        Ok(false) => photos_error(&state, id, &headers, "Фото не найдено").await,
        Err(error) => {
            tracing::error!(error = %error, "Не удалось удалить фото (веб-админка)");
            photos_error(&state, id, &headers, "Не удалось удалить фото").await
        }
    }
}

/// POST /admin/cars/:id/images/cover — сделать фото обложкой.
pub async fn set_cover(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Form(form): Form<ImageForm>,
) -> Response {
    let Some(url) = normalize_url(&form.url) else {
        return photos_error(&state, id, &headers, "Некорректный путь фото").await;
    };
    let Some(car) = fetch_car(&state, id).await else {
        return error_page(
            StatusCode::NOT_FOUND,
            "Автомобиль не найден",
            "Такой записи нет в каталоге.",
        );
    };
    if !car.images.iter().any(|image| image == &url) {
        return photos_error(&state, id, &headers, "Фото не найдено в галерее").await;
    }

    let mut ordered: Vec<String> = car
        .images
        .iter()
        .filter(|image| *image != &url)
        .cloned()
        .collect();
    ordered.insert(0, url);
    reorder_and_render(&state, id, &headers, &ordered, "Обложка изменена").await
}

/// POST /admin/cars/:id/images/move — сдвинуть фото выше/ниже.
pub async fn move_image(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Form(form): Form<ImageForm>,
) -> Response {
    let Some(url) = normalize_url(&form.url) else {
        return photos_error(&state, id, &headers, "Некорректный путь фото").await;
    };
    let Some(car) = fetch_car(&state, id).await else {
        return error_page(
            StatusCode::NOT_FOUND,
            "Автомобиль не найден",
            "Такой записи нет в каталоге.",
        );
    };
    let Some(position) = car.images.iter().position(|image| image == &url) else {
        return photos_error(&state, id, &headers, "Фото не найдено в галерее").await;
    };

    let target = match form.direction.as_str() {
        "up" if position > 0 => position - 1,
        "down" if position + 1 < car.images.len() => position + 1,
        _ => position,
    };

    let mut ordered = car.images.clone();
    ordered.swap(position, target);
    reorder_and_render(&state, id, &headers, &ordered, "Порядок фото изменён").await
}

/// POST /admin/ai/description — сгенерировать текст объявления.
pub async fn ai_description(State(state): State<AppState>, Form(form): Form<CarForm>) -> Response {
    let car = match form.to_new_car() {
        Ok(car) => car,
        Err(errors) => {
            return render_template(&AiDescriptionTemplate {
                text: None,
                error: Some(
                    errors
                        .into_iter()
                        .map(|e| e.message)
                        .collect::<Vec<_>>()
                        .join("; "),
                ),
                model: String::new(),
            })
        }
    };

    let request = DescribeCarRequest {
        brand: car.brand,
        model: car.model,
        year: car.year,
        mileage: car.mileage,
        engine: car.engine,
        transmission: car.transmission,
        drive: car.drive,
        price_rub: car.price_rub,
    };

    match describe_car(&state, &request).await {
        Ok((text, model)) => render_template(&AiDescriptionTemplate {
            text: Some(text),
            error: None,
            model,
        }),
        Err((_, Json(response))) => render_template(&AiDescriptionTemplate {
            text: None,
            error: Some(response.message),
            model: String::new(),
        }),
    }
}

// ── вспомогательные функции ────────────────────────────────────────────────

/// Проверяет URL фото из формы: только наши файлы внутри `/images/`.
fn normalize_url(raw: &str) -> Option<String> {
    let url = raw.trim();
    if url.starts_with(IMAGES_PREFIX) && !url.contains("..") {
        Some(url.to_string())
    } else {
        None
    }
}

/// Переставляет галерею и возвращает обновлённый фрагмент.
async fn reorder_and_render(
    state: &AppState,
    id: i64,
    headers: &HeaderMap,
    ordered: &[String],
    message: &str,
) -> Response {
    match state.db.reorder_car_images(id, ordered).await {
        Ok(true) => photos_success(state, id, headers, message.to_string()).await,
        Ok(false) => photos_error(state, id, headers, "Не удалось изменить порядок фото").await,
        Err(error) => {
            tracing::error!(error = %error, "Не удалось переставить фото");
            photos_error(state, id, headers, "Не удалось изменить порядок фото").await
        }
    }
}

/// Успешный ответ по фотографиям: галерея + тост.
async fn photos_success(
    state: &AppState,
    id: i64,
    headers: &HeaderMap,
    message: String,
) -> Response {
    if !is_htmx(headers) {
        return redirect(headers, &format!("/admin/cars/{id}"));
    }
    match state.db.car_by_id(id).await {
        Ok(Some(car)) => {
            let main = render_html(&PhotosGridTemplate { car });
            html_response(with_toast(main, "ok", &message))
        }
        Ok(None) => error_page(
            StatusCode::NOT_FOUND,
            "Автомобиль не найден",
            "Такой записи нет в каталоге.",
        ),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить авто после изменения фото");
            error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Ошибка базы данных",
                "Не удалось обновить галерею.",
            )
        }
    }
}

/// Ошибка по фотографиям: текущая галерея + тост об ошибке.
async fn photos_error(state: &AppState, id: i64, headers: &HeaderMap, message: &str) -> Response {
    if !is_htmx(headers) {
        return redirect(headers, &format!("/admin/cars/{id}"));
    }
    match state.db.car_by_id(id).await {
        Ok(Some(car)) => {
            let main = render_html(&PhotosGridTemplate { car });
            html_response(with_toast(main, "error", message))
        }
        Ok(None) => error_page(
            StatusCode::NOT_FOUND,
            "Автомобиль не найден",
            "Такой записи нет в каталоге.",
        ),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить авто после ошибки фото");
            error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Ошибка базы данных",
                "Не удалось обновить галерею.",
            )
        }
    }
}

/// Ошибка статуса: карточка статуса + тост.
async fn status_error(state: &AppState, id: i64, headers: &HeaderMap, message: &str) -> Response {
    if !is_htmx(headers) {
        return redirect(headers, &format!("/admin/cars/{id}"));
    }
    match state.db.car_by_id(id).await {
        Ok(Some(car)) => {
            let main = render_html(&StatusCardTemplate { car });
            html_response(with_toast(main, "error", message))
        }
        Ok(None) => error_page(
            StatusCode::NOT_FOUND,
            "Автомобиль не найден",
            "Такой записи нет в каталоге.",
        ),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить авто после ошибки статуса");
            error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Ошибка базы данных",
                "Не удалось обновить статус.",
            )
        }
    }
}

/// Ответ формы с ошибками: фрагмент для htmx или полная страница без JS.
fn form_errors(
    form: &CarForm,
    errors: Vec<FieldError>,
    is_edit: bool,
    action: String,
    car_id: Option<i64>,
    headers: &HeaderMap,
) -> Response {
    if is_htmx(headers) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            render_template(&CarFormFragmentTemplate {
                form: form.clone(),
                errors,
                is_edit,
                action,
                car_id,
            }),
        )
            .into_response();
    }

    (
        StatusCode::UNPROCESSABLE_ENTITY,
        render_template(&CarFormPageTemplate {
            page_title: if is_edit {
                "Редактирование"
            } else {
                "Новый автомобиль"
            },
            nav: if is_edit { "cars" } else { "new" },
            form: form.clone(),
            errors,
            is_edit,
            action,
            car_id,
        }),
    )
        .into_response()
}

/// Редирект: `HX-Redirect` для htmx, `303` для обычной формы.
fn redirect(headers: &HeaderMap, to: &str) -> Response {
    if is_htmx(headers) {
        Response::builder()
            .status(StatusCode::OK)
            .header(HX_REDIRECT, to)
            .body(Body::empty())
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
    } else {
        Redirect::to(to).into_response()
    }
}

/// Добавляет к HTML-фрагменту OOB-тост.
fn with_toast(main: String, kind: &'static str, message: &str) -> String {
    let toast = match (ToastTemplate {
        kind,
        message: message.to_string(),
    })
    .render()
    {
        Ok(html) => html,
        Err(error) => {
            tracing::error!(error = %error, "Не удалось отрисовать тост");
            String::new()
        }
    };
    format!("{main}{toast}")
}

/// Собирает HTML-ответ из готовой строки.
fn html_response(html: String) -> Response {
    Html(html).into_response()
}

/// Рендерит шаблон в строку; при ошибке — безопасная заглушка.
fn render_html<T: Template>(template: &T) -> String {
    match template.render() {
        Ok(html) => html,
        Err(error) => {
            tracing::error!(error = %error, "Не удалось отрисовать фрагмент админки");
            "<div class=\"a-alert a-alert-error\">Ошибка шаблона</div>".to_string()
        }
    }
}

/// Рендерит шаблон в `Response`.
fn render_template<T: Template>(template: &T) -> Response {
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(error) => {
            tracing::error!(error = %error, "Не удалось отрисовать фрагмент админки");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<div class=\"a-alert a-alert-error\">Ошибка шаблона</div>".to_string()),
            )
                .into_response()
        }
    }
}

/// Читает автомобиль или `None`.
async fn fetch_car(state: &AppState, id: i64) -> Option<Car> {
    match state.db.car_by_id(id).await {
        Ok(car) => car,
        Err(error) => {
            tracing::error!(error = %error, "Не удалось получить автомобиль");
            None
        }
    }
}
