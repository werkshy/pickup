/**
 * Page handlers: full server-rendered pages. Every page is also what htmx
 * fetches for boosted navigation; htmx carves the `#content` element out of
 * the response (see the hx-select on <body> in base.html), so one handler
 * serves both full loads and swaps.
 */
use std::path::Path;

use actix_web::{get, web, HttpRequest, HttpResponse};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::filemanager::collection_builder::DEFAULT_CATEGORY;
use crate::filemanager::utils::generate_category_id;
use crate::webui::render;
use crate::webui::views::{FolderPageView, FolderTemplate, HomeTemplate, Shell};

/// The home page: the default ("Music") category, so the collection's main
/// folders are what you land on. If that category isn't in the collection,
/// fall back to the list of every category.
#[get("/")]
pub async fn home(data: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse, AppError> {
    let id = generate_category_id(Path::new(DEFAULT_CATEGORY));
    let collection = data.collection.get();
    match collection.get_folder(&id) {
        Some(folder) => {
            let page = FolderPageView::from_folder(folder);
            folder_page(&data, &req, page).await
        }
        None => {
            let shell = Shell::for_page(&data, &req, "Pickup", "").await;
            render(&HomeTemplate { shell })
        }
    }
}

/// A folder page: category, artist, album or disc. Every level renders the
/// same way - folders below it, tracks inside it - so one route serves the
/// whole tree. An ID that is not in the collection gets the HTML error
/// page, since this route is only reached by browsers.
#[get("/browse/{id}")]
pub async fn browse(
    data: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let id = id.into_inner();
    let collection = data.collection.get();
    let folder = match collection.get_folder(&id) {
        Some(folder) => folder,
        None => {
            return Ok(crate::webui::not_found_page(&data, &req, &format!("/browse/{id}")).await)
        }
    };
    let page = FolderPageView::from_folder(folder);
    folder_page(&data, &req, page).await
}

/// Render a folder with the full page shell. Shared by the home page (which
/// lands on the default category) and the browse route.
async fn folder_page(
    data: &AppState,
    req: &HttpRequest,
    page: FolderPageView,
) -> Result<HttpResponse, AppError> {
    let title = if page.heading.is_empty() {
        "Pickup".to_string()
    } else {
        format!("{} · Pickup", page.heading)
    };
    let shell = Shell::for_page(data, req, &title, &page.active_category).await;
    render(&FolderTemplate { shell, page })
}
