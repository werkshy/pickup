/**
 * The web UI: server-rendered pages with htmx for navigation and actions,
 * served alongside the JSON API (which stays untouched for programmatic
 * use).
 *
 * These handlers are deliberately plain actix services, not registered
 * through the OpenAPI-collecting `api::register_services`: they are for
 * browsers, not API consumers, and the generated spec describes JSON.
 */
pub mod actions;
pub mod pages;
pub mod views;

use std::path::Path;

use actix_files::Files;
use actix_web::dev::{ServiceFactory, ServiceRequest};
use actix_web::http::StatusCode;
use actix_web::{Error, HttpResponse, ResponseError};
use askama::Template;
use utoipa_actix_web::UtoipaApp;

use crate::app_state::AppState;
use crate::error::AppError;

/// Static assets (htmx, CSS, JS), served from disk so they can be edited
/// without a rebuild.
/// TODO: Embed in binary for release builds.
const STATIC_DIR: &str = "frontend/static";

/**
 * Register the web UI services. The JSON API's `UtoipaApp::service` only
 * accepts OpenAPI-documented handlers, so the plain (undocumented) web UI
 * services go onto the wrapped actix App through `map`.
 */
pub fn register<T>(app: UtoipaApp<T>) -> UtoipaApp<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    app.map(|app| {
        app.service(pages::home)
            .service(pages::browse)
            .service(actions::play)
            .service(actions::add)
            .service(actions::next)
            .service(actions::remove)
            .service(actions::move_track)
            .service(actions::clear)
            .service(actions::queue_fragment)
            .service(Files::new("/static", static_dir()))
    })
}

fn static_dir() -> &'static Path {
    Path::new(STATIC_DIR)
}

/**
 * Render a template into a HTML response.
 */
pub(crate) fn render<T: Template>(template: &T) -> Result<HttpResponse, AppError> {
    render_with_status(StatusCode::OK, template)
}

pub(crate) fn render_with_status<T: Template>(
    status: StatusCode,
    template: &T,
) -> Result<HttpResponse, AppError> {
    let body = template
        .render()
        .map_err(|error| AppError::internal(format!("template render failed: {error}")))?;
    Ok(HttpResponse::build(status)
        .content_type("text/html; charset=utf-8")
        .body(body))
}

/**
 * The HTML 404 page for browser navigation: a full page with the usual
 * shell, so it works as a direct page load and as a boosted swap (where
 * only #content is carved out of it). API consumers keep the JSON error
 * shape - the content negotiation lives at the app-wide fallback.
 */
pub async fn not_found_page(state: &AppState, path: &str) -> HttpResponse {
    let shell = views::Shell::for_page(state, "Not found · Pickup", "").await;
    match render_with_status(
        StatusCode::NOT_FOUND,
        &views::NotFoundTemplate {
            shell,
            path: path.to_string(),
        },
    ) {
        Ok(response) => response,
        // If even the error page fails to render, fall back to the JSON shape.
        Err(_) => AppError::not_found("No such route").error_response(),
    }
}
