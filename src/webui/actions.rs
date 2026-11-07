/**
 * Action handlers: playback and queue changes issued by the web UI. They
 * share the JSON API's operations (api::queue) and talk to the PlayerClient
 * directly, so both entry points behave identically. Every response carries
 * the refreshed queue drawer and player bar as out-of-band swaps: htmx
 * applies the OOB content and leaves the rest of the page alone, so a click
 * updates the chrome with no polling and no page change.
 */
use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;

use crate::api::queue::{add_to_queue, clear_queue, move_in_queue, remove_from_queue};
use crate::api::types::{ApiQueueInput, ApiQueuePosition};
use crate::app_state::AppState;
use crate::error::AppError;
use crate::webui::render;
use crate::webui::views::{ActionTemplate, PlayerBarView, ProgressTemplate, Shell};

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
    action_response(&data).await
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
    action_response(&data).await
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
    action_response(&data).await
}

/// Remove one track from the queue by position.
#[post("/web/queue/remove/{index}")]
pub async fn remove(
    data: web::Data<AppState>,
    index: web::Path<usize>,
) -> Result<HttpResponse, AppError> {
    remove_from_queue(&data, index.into_inner()).await?;
    action_response(&data).await
}

/// Move a track within the queue (reorder).
#[post("/web/queue/move/{from}/{to}")]
pub async fn move_track(
    data: web::Data<AppState>,
    path: web::Path<(usize, usize)>,
) -> Result<HttpResponse, AppError> {
    let (from, to) = path.into_inner();
    move_in_queue(&data, from, to).await?;
    action_response(&data).await
}

/// Stop playback and empty the queue.
#[post("/web/queue/clear")]
pub async fn clear(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    clear_queue(&data).await?;
    action_response(&data).await
}

/// The transport play button: resume a paused track, or start the front of
/// the queue when idle. A played-out queue is not an error here - clicking
/// play when there is nothing to play just changes nothing.
#[post("/web/resume")]
pub async fn resume(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let _outcome = data
        .player
        .play_queue_if_idle()
        .await
        .map_err(AppError::from)?;
    action_response(&data).await
}

/// The transport pause button: pause the sounding track. No-op when idle.
#[post("/web/pause")]
pub async fn pause(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    data.player.pause().await.map_err(AppError::from)?;
    action_response(&data).await
}

/// The transport skip button: next track in the queue. No-op when idle.
#[post("/web/skip")]
pub async fn skip(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    data.player.skip().await.map_err(AppError::from)?;
    action_response(&data).await
}

/// The transport stop button: stop playback, keep the queue. Resume starts
/// the front of the queue again.
#[post("/web/stop")]
pub async fn stop(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    data.player.stop().await.map_err(AppError::from)?;
    action_response(&data).await
}

/// The volume slider: the form-encoded `volume` value, clamped server-side.
/// The response's out-of-band bar carries the value that was actually set,
/// so the slider always ends up where the server says.
#[derive(Deserialize)]
pub struct VolumeForm {
    volume: i64,
}

#[post("/web/volume")]
pub async fn volume(
    data: web::Data<AppState>,
    form: web::Form<VolumeForm>,
) -> Result<HttpResponse, AppError> {
    let clamped = form.volume.clamp(0, 100) as u8;
    let _set = data
        .player
        .set_volume(clamped)
        .await
        .map_err(AppError::from)?;
    action_response(&data).await
}

/// The queue drawer's list on its own, for direct swaps into the drawer
/// (e.g. a drawer-focused refresh).
#[get("/fragments/queue")]
pub async fn queue_fragment(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let queue = crate::webui::views::QueueView::build(&data).await;
    render(&crate::webui::views::QueueTemplate {
        shell: Shell::for_fragment(queue),
    })
}

/**
 * The refreshed queue drawer and player bar as an out-of-band swap response.
 */
async fn action_response(data: &web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let shell = Shell::for_action(data).await;
    render(&ActionTemplate { shell })
}

/**
 * The progress readout alone: swapped in place by the bar's one-second poll.
 * The whole bar is deliberately not re-rendered here, so polling never
 * disturbs user interaction with the bar (e.g. dragging the volume slider).
 */
#[get("/web/fragments/progress")]
pub async fn progress(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let snapshot = crate::webui::views::player_snapshot(&data).await;
    let bar = PlayerBarView::from_snapshot(&data, &snapshot);
    render(&ProgressTemplate { bar })
}
