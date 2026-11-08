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

use actix_web::dev::{ServiceFactory, ServiceRequest};
use actix_web::http::StatusCode;
use actix_web::{
    cookie::time::Duration as CookieDuration, cookie::Cookie, get, web, Error, HttpRequest,
    HttpResponse, ResponseError,
};
use askama::Template;
use rust_embed::RustEmbed;
use utoipa_actix_web::UtoipaApp;

use crate::app_state::AppState;
use crate::error::AppError;

/**
 * The two playback modes, chosen per browser with the `pickup_mode` cookie.
 * Browser mode plays in the browser through its own session queue; jukebox
 * mode plays on the server through the global queue. The default is browser
 * mode, and unknown cookie values fall back to it.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackMode {
    Browser,
    Jukebox,
}

pub const MODE_COOKIE: &str = "pickup_mode";
const BROWSER_MODE: &str = "browser";
const JUKEBOX_MODE: &str = "jukebox";
const MODE_TTL_DAYS: i64 = 30;

impl PlaybackMode {
    /// The value the mode cookie carries.
    pub fn cookie_value(self) -> &'static str {
        match self {
            PlaybackMode::Browser => BROWSER_MODE,
            PlaybackMode::Jukebox => JUKEBOX_MODE,
        }
    }

    /// The name a mode is known by in the UI (labels and data attributes).
    pub fn name(self) -> &'static str {
        self.cookie_value()
    }
}

/// Read the request's playback mode, defaulting to browser mode.
pub fn playback_mode(req: &HttpRequest) -> PlaybackMode {
    match req.cookie(MODE_COOKIE) {
        Some(cookie) if cookie.value() == JUKEBOX_MODE => PlaybackMode::Jukebox,
        _ => PlaybackMode::Browser,
    }
}

/// The cookie that persists the playback mode choice per browser.
pub fn mode_cookie(mode: PlaybackMode) -> Cookie<'static> {
    Cookie::build(MODE_COOKIE, mode.cookie_value())
        .path("/")
        .max_age(CookieDuration::days(MODE_TTL_DAYS))
        .same_site(actix_web::cookie::SameSite::Lax)
        .finish()
}

/// Static assets (htmx, CSS, JS, favicons). In debug builds `rust-embed`
/// serves them from disk, in release builds they are compiled into the binary.
#[derive(RustEmbed)]
#[folder = "frontend/static/"]
struct StaticAssets;

/// Serve a static asset by name. Reject `..` path components, especially
/// important for debug builds that are reading from the filesystem.
#[get("/static/{filename:.*}")]
async fn static_asset(filename: web::Path<String>) -> HttpResponse {
    let filename = filename.into_inner();
    if filename
        .split(['/', '\\'])
        .any(|component| component == "..")
    {
        return HttpResponse::NotFound().finish();
    }
    match StaticAssets::get(&filename) {
        Some(asset) => {
            let mime = mime_guess::from_path(&filename).first_or_octet_stream();
            HttpResponse::Ok()
                .content_type(mime.as_ref())
                .body(asset.data.into_owned())
        }
        None => HttpResponse::NotFound().finish(),
    }
}

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
            .service(actions::resume)
            .service(actions::pause)
            .service(actions::skip)
            .service(actions::stop)
            .service(actions::volume)
            .service(actions::mode_browser)
            .service(actions::mode_jukebox)
            .service(actions::queue_fragment)
            .service(actions::progress)
            .service(static_asset)
    })
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
pub async fn not_found_page(state: &AppState, req: &HttpRequest, path: &str) -> HttpResponse {
    let shell = views::Shell::for_page(state, req, "Not found · Pickup", "").await;
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
