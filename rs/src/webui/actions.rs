/**
 * Action handlers: playback and queue changes issued by the web UI. They
 * share the JSON API's operations (api::queue), so both entry points behave
 * identically, and respond with the refreshed queue drawer as an
 * out-of-band swap: htmx applies the OOB content and leaves the rest of the
 * page alone, so the drawer updates without any polling.
 */
use actix_web::{get, post, web, HttpResponse};

use crate::api::queue::{add_to_queue, clear_queue, move_in_queue, remove_from_queue};
use crate::api::types::{ApiQueueInput, ApiQueuePosition};
use crate::app_state::AppState;
use crate::error::AppError;
use crate::webui::render;
use crate::webui::views::{QueueTemplate, Shell};

/// Play a track or everything under a folder right now: replaces the queue
/// with it and starts playing.
#[post("/web/play/{id}")]
pub async fn play(
    data: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let input = ApiQueueInput {
        id: id.into_inner(),
        position: ApiQueuePosition::Replace,
        play_now: true,
    };
    add_to_queue(&data, &input).await?;
    queue_response(&data).await
}

/// Add a track or folder to the back of the queue.
#[post("/web/queue/add/{id}")]
pub async fn add(
    data: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let input = ApiQueueInput {
        id: id.into_inner(),
        position: ApiQueuePosition::Tail,
        play_now: false,
    };
    add_to_queue(&data, &input).await?;
    queue_response(&data).await
}

/// Queue a track or folder to play next, behind the current track.
#[post("/web/queue/next/{id}")]
pub async fn next(
    data: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let input = ApiQueueInput {
        id: id.into_inner(),
        position: ApiQueuePosition::Next,
        play_now: false,
    };
    add_to_queue(&data, &input).await?;
    queue_response(&data).await
}

/// Remove one track from the queue by position.
#[post("/web/queue/remove/{index}")]
pub async fn remove(
    data: web::Data<AppState>,
    index: web::Path<usize>,
) -> Result<HttpResponse, AppError> {
    remove_from_queue(&data, index.into_inner()).await?;
    queue_response(&data).await
}

/// Move a track within the queue (reorder).
#[post("/web/queue/move/{from}/{to}")]
pub async fn move_track(
    data: web::Data<AppState>,
    path: web::Path<(usize, usize)>,
) -> Result<HttpResponse, AppError> {
    let (from, to) = path.into_inner();
    move_in_queue(&data, from, to).await?;
    queue_response(&data).await
}

/// Stop playback and empty the queue.
#[post("/web/queue/clear")]
pub async fn clear(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    clear_queue(&data).await?;
    queue_response(&data).await
}

/// The queue drawer's list on its own, for direct swaps into the drawer
/// (e.g. polling fallback later).
#[get("/fragments/queue")]
pub async fn queue_fragment(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let queue = crate::webui::views::QueueView::build(&data).await;
    render(&QueueTemplate {
        shell: Shell::for_fragment(queue),
    })
}

/**
 * The refreshed queue drawer as an out-of-band swap response.
 */
async fn queue_response(data: &web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let queue = crate::webui::views::QueueView::build(data).await;
    render(&QueueTemplate {
        shell: Shell::for_action(queue),
    })
}
