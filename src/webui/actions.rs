/**
 * Action handlers: playback and queue changes issued by the web UI. They
 * share the JSON API's operations and talk to the PlayerClient (jukebox
 * mode) or the session's queue (browser mode), so every entry point behaves
 * the same. Every response carries the refreshed queue drawer and player
 * bar as out-of-band swaps: htmx applies the OOB content and leaves the rest
 * of the page alone, so a click updates the chrome with no polling and no
 * page change.
 */
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;

use crate::api::queue::{add_to_queue, clear_queue, move_in_queue, remove_from_queue};
use crate::api::session::{session_add, session_clear, session_move, session_remove};
use crate::api::types::{ApiQueueInput, ApiQueuePosition};
use crate::app_state::AppState;
use crate::error::AppError;
use crate::session::request_session_id;
use crate::webui::render;
use crate::webui::views::{ActionTemplate, PlayerBarView, ProgressTemplate, Shell};
use crate::webui::{mode_cookie, playback_mode, PlaybackMode};

/// Play a track or everything under a folder right now: replaces the queue
/// with it and starts playing.
#[post("/web/play/{id}")]
pub async fn play(
    data: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let input = ApiQueueInput {
        id: id.into_inner(),
        position: ApiQueuePosition::Replace,
        play_now: true,
    };
    apply_queue_add(&data, &req, &input).await?;
    action_response(&data, &req, true).await
}

/// Add a track or folder to the back of the queue.
#[post("/web/queue/add/{id}")]
pub async fn add(
    data: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let input = ApiQueueInput {
        id: id.into_inner(),
        position: ApiQueuePosition::Tail,
        play_now: false,
    };
    apply_queue_add(&data, &req, &input).await?;
    action_response(&data, &req, false).await
}

/// Queue a track or folder to play next, behind the current track.
#[post("/web/queue/next/{id}")]
pub async fn next(
    data: web::Data<AppState>,
    req: HttpRequest,
    id: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let input = ApiQueueInput {
        id: id.into_inner(),
        position: ApiQueuePosition::Next,
        play_now: false,
    };
    apply_queue_add(&data, &req, &input).await?;
    action_response(&data, &req, false).await
}

/**
 * Add to whichever queue the browser's playback mode drives: the session's
 * queue in browser mode, the jukebox queue in jukebox mode.
 */
async fn apply_queue_add(
    data: &web::Data<AppState>,
    req: &HttpRequest,
    input: &ApiQueueInput,
) -> Result<(), AppError> {
    match playback_mode(req) {
        PlaybackMode::Jukebox => add_to_queue(data, input).await,
        PlaybackMode::Browser => {
            let session = request_session_id(req)?;
            session_add(data, &session, input)
        }
    }
}

/// Remove one track from the queue by position.
#[post("/web/queue/remove/{index}")]
pub async fn remove(
    data: web::Data<AppState>,
    req: HttpRequest,
    index: web::Path<usize>,
) -> Result<HttpResponse, AppError> {
    let index = index.into_inner();
    match playback_mode(&req) {
        PlaybackMode::Jukebox => remove_from_queue(&data, index).await?,
        PlaybackMode::Browser => {
            let session = request_session_id(&req)?;
            session_remove(&data, &session, index)?;
        }
    }
    action_response(&data, &req, false).await
}

/// Move a track within the queue (reorder).
#[post("/web/queue/move/{from}/{to}")]
pub async fn move_track(
    data: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<(usize, usize)>,
) -> Result<HttpResponse, AppError> {
    let (from, to) = path.into_inner();
    match playback_mode(&req) {
        PlaybackMode::Jukebox => move_in_queue(&data, from, to).await?,
        PlaybackMode::Browser => {
            let session = request_session_id(&req)?;
            session_move(&data, &session, from, to)?;
        }
    }
    action_response(&data, &req, false).await
}

/// Stop playback and empty the queue.
#[post("/web/queue/clear")]
pub async fn clear(data: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse, AppError> {
    match playback_mode(&req) {
        PlaybackMode::Jukebox => clear_queue(&data).await?,
        PlaybackMode::Browser => {
            let session = request_session_id(&req)?;
            session_clear(&data, &session)?;
        }
    }
    action_response(&data, &req, false).await
}

/// The transport play button: resume a paused track, or start the front of
/// the queue when idle. A played-out queue is not an error here - clicking
/// play when there is nothing to play just changes nothing. In browser mode
/// the transport is the browser's own player, which never calls this.
#[post("/web/resume")]
pub async fn resume(data: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse, AppError> {
    let _outcome = data
        .player
        .play_queue_if_idle()
        .await
        .map_err(AppError::from)?;
    action_response(&data, &req, false).await
}

/// The transport pause button: pause the sounding track. No-op when idle.
#[post("/web/pause")]
pub async fn pause(data: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse, AppError> {
    data.player.pause().await.map_err(AppError::from)?;
    action_response(&data, &req, false).await
}

/// The transport skip button: next track in the queue. No-op when idle.
#[post("/web/skip")]
pub async fn skip(data: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse, AppError> {
    data.player.skip().await.map_err(AppError::from)?;
    action_response(&data, &req, false).await
}

/// The transport stop button: stop playback, keep the queue. Resume starts
/// the front of the queue again.
#[post("/web/stop")]
pub async fn stop(data: web::Data<AppState>, req: HttpRequest) -> Result<HttpResponse, AppError> {
    data.player.stop().await.map_err(AppError::from)?;
    action_response(&data, &req, false).await
}

/// The volume slider: the form-encoded `volume` value, clamped server-side.
/// The response's out-of-band bar carries the value that was actually set,
/// so the slider always ends up where the server says. In browser mode the
/// volume is the browser player's own, so this is never called.
#[derive(Deserialize)]
pub struct VolumeForm {
    volume: i64,
}

#[post("/web/volume")]
pub async fn volume(
    data: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<VolumeForm>,
) -> Result<HttpResponse, AppError> {
    let clamped = form.volume.clamp(0, 100) as u8;
    let _set = data
        .player
        .set_volume(clamped)
        .await
        .map_err(AppError::from)?;
    action_response(&data, &req, false).await
}

/**
 * Switch the browser's playback mode (the header's two-state toggle posts
 * the mode it should switch to): the mode goes into the response's cookie,
 * and the chrome (mode switch, drawer and bar) is re-rendered for that
 * mode, so switching needs no page reload. Switching to browser mode also
 * stops the server player: its playback would otherwise keep sounding with
 * no controls anywhere in the UI. (Switching to jukebox mode stops the
 * browser player the same way, in the player script.)
 */
#[post("/web/mode/browser")]
pub async fn mode_browser(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    set_mode(&data, &req, PlaybackMode::Browser).await
}

#[post("/web/mode/jukebox")]
pub async fn mode_jukebox(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    set_mode(&data, &req, PlaybackMode::Jukebox).await
}

async fn set_mode(
    data: &web::Data<AppState>,
    req: &HttpRequest,
    mode: PlaybackMode,
) -> Result<HttpResponse, AppError> {
    if playback_mode(req) == mode {
        // Already there: just re-render the chrome, so the mode switch and
        // the drawer are in step even after a stale render.
        return action_response(data, req, false).await;
    }
    if mode == PlaybackMode::Browser {
        // Nothing is playing from the browser yet after a switch; stopping
        // an idle player is a no-op.
        let _ = data.player.stop().await;
    }
    let shell = Shell::for_action_in_mode(data, req, mode, false).await;
    let mut response = render(&ActionTemplate { shell })?;
    let _ = response.add_cookie(&mode_cookie(mode));
    Ok(response)
}

/**
 * The queue drawer's list on its own, for direct swaps into the drawer
 * (e.g. after the browser player advances the session queue).
 */
#[get("/fragments/queue")]
pub async fn queue_fragment(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let queue = crate::webui::views::QueueView::build(&data, &req).await;
    render(&crate::webui::views::QueueTemplate {
        shell: Shell::for_fragment(queue),
    })
}

/**
 * The refreshed queue drawer and player bar as an out-of-band swap response,
 * rendered for the request's playback mode. `start_playing` tells the
 * browser player (browser mode) to start the session queue's front track.
 */
async fn action_response(
    data: &web::Data<AppState>,
    req: &HttpRequest,
    start_playing: bool,
) -> Result<HttpResponse, AppError> {
    let shell = Shell::for_action(data, req, start_playing).await;
    render(&ActionTemplate { shell })
}

/**
 * The progress readout alone: swapped in place by the bar's one-second poll.
 * The whole bar is deliberately not re-rendered here, so polling never
 * disturbs user interaction with the bar (e.g. dragging the volume slider).
 * The poll only exists in jukebox mode; browser mode's progress lives in
 * the browser.
 */
#[get("/web/fragments/progress")]
pub async fn progress(data: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let snapshot = crate::webui::views::player_snapshot(&data).await;
    let bar = PlayerBarView::from_snapshot(&data, &snapshot);
    render(&ProgressTemplate { bar })
}
