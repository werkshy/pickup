/**
 * Page handlers: full server-rendered pages. Every page is also what htmx
 * fetches for boosted navigation; htmx carves the `#content` element out of
 * the response (see the hx-select on <body> in base.html), so one handler
 * serves both full loads and swaps.
 */
use actix_web::{get, web, HttpResponse};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::webui::render;
use crate::webui::views::{FolderPageView, FolderTemplate, HomeTemplate, Shell};

/// The home page: the collection's categories.
#[get("/")]
pub async fn home(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let shell = Shell::for_page(&data, "Pickup", "").await;
    render(&HomeTemplate { shell })
}

/// A folder page: category, artist, album or disc. Every level renders the
/// same way - folders below it, tracks inside it - so one route serves the
/// whole tree. An ID that is not in the collection gets the HTML error
/// page, since this route is only reached by browsers.
#[get("/browse/{id}")]
pub async fn browse(
    data: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = id.into_inner();
    let folder = match data.collection.get_folder(&id) {
        Some(folder) => folder,
        None => return Ok(crate::webui::not_found_page(&data, &format!("/browse/{id}")).await),
    };
    let page = FolderPageView::from_folder(folder);

    let title = if page.heading.is_empty() {
        "Pickup".to_string()
    } else {
        format!("{} · Pickup", page.heading)
    };
    let shell = Shell::for_page(&data, &title, &page.active_category).await;
    render(&FolderTemplate { shell, page })
}
