/**
 * Collection refresh from the browser. Starting one only kicks off the
 * background rebuild; the status fragment polls until the new collection has
 * been swapped in, so the page never blocks on a slow music dir.
 */
use actix_web::{get, post, web, HttpResponse};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::webui::render;
use crate::webui::views::{RefreshTemplate, RefreshView};

/// Start a collection refresh (or report the one already running).
#[post("/web/refresh")]
pub async fn trigger(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    data.refresh.start();
    response(&data).await
}

/// The refresh control's current state, for the status poll.
#[get("/web/fragments/refresh")]
pub async fn fragment(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    response(&data).await
}

async fn response(data: &web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let refresh = RefreshView::from_status(data.refresh.status());
    render(&RefreshTemplate { refresh })
}
