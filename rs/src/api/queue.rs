use actix_web::{get, http::StatusCode, post, web, Responder, Result};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::filemanager::dto::CollectionLocation;
use crate::{api::list::ApiTrack, app_state::AppState};

// TODO extract 'ApiCollectionLocaltion out of here
#[derive(Deserialize, Debug)]
struct ApiQueueInput {
    category: String,
    artist: Option<String>,
    album: Option<String>,
    disc: Option<String>,
    track: Option<String>,
    #[serde(default)] // Defaults to false
    clear: bool,
}

#[derive(Serialize, Debug)]
struct ApiQueueResponse {
    tracks: Vec<ApiTrack>,
    position: usize,
}

#[post("/queue/add")]
pub async fn add(
    data: web::Data<AppState>,
    input: web::Json<ApiQueueInput>,
) -> Result<impl Responder, AppError> {
    let collection_location = CollectionLocation {
        category: input.category.clone(),
        artist: input.artist.clone(),
        album: input.album.clone(),
        disc: input.disc.clone(),
        track: input.track.clone(),
    };

    let maybe_tracks = data.collection.get_tracks_under(&collection_location);
    if maybe_tracks.is_none() {
        return Err(AppError::new(
            "No matching music found in the collection",
            StatusCode::NOT_FOUND,
        ));
    }
    let tracks = maybe_tracks.unwrap();

    log::info!("Adding {} track to playlist", tracks.len());
    let mut queue = data.queue.write().unwrap();
    if input.clear {
        queue.clear();
    }
    queue.add_tracks(tracks.into_iter().cloned().collect());
    Ok(web::Json(ApiQueueResponse {
        tracks: queue.tracks.iter().map(ApiTrack::from_track).collect(),
        position: queue.position,
    }))
}

#[post("/queue/clear")]
pub async fn clear(data: web::Data<AppState>) -> impl Responder {
    let mut queue = data.queue.write().unwrap();
    queue.clear();
    web::Json(ApiQueueResponse {
        tracks: queue.tracks.iter().map(ApiTrack::from_track).collect(),
        position: queue.position,
    })
}

#[get("/queue")]
pub async fn get_queue(data: web::Data<AppState>) -> impl Responder {
    let queue = data.queue.read().unwrap();
    web::Json(ApiQueueResponse {
        tracks: queue.tracks.iter().map(ApiTrack::from_track).collect(),
        position: queue.position,
    })
}
