use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::filemanager::model::Track;

#[derive(Debug, Serialize, ToSchema)]
pub struct IndexResponse {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StatusResponse {
    pub status: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VolumeResponse {
    pub volume: u8,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiCategory {
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiTrack {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub disc: Option<String>,
    pub category: String,
}

impl ApiTrack {
    pub fn from_track(track: &Track) -> Self {
        ApiTrack {
            id: track.id.clone(),
            title: track.name.clone(),
            artist: track.artist.clone(),
            album: track.album.clone(),
            disc: track.disc.clone(),
            category: track.category.clone(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListCategoriesResponse {
    pub categories: Vec<ApiCategory>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ApiQueueInput {
    pub category: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub disc: Option<String>,
    pub track: Option<String>,
    #[serde(default)] // Defaults to false
    pub clear: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiQueueResponse {
    pub tracks: Vec<ApiTrack>,
    pub position: usize,
}

/// The body of every error response, so consumers can rely on one shape.
///
/// Serialized from `crate::error::AppError`, which carries the same data
/// alongside the status code it was built with.
///
/// ```json
/// { "code": "not_found", "message": "No matching music found in the collection" }
/// ```
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub code: ApiErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiErrorCode {
    NotFound,
    InvalidRequest,
    PlayerUnavailable,
    PlayerTimeout,
    PlaybackFailed,
    Internal,
}
