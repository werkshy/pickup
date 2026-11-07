use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::filemanager::model::{Folder, Kind, Track};
use crate::player::PlaybackStatus;

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
pub struct PlaybackStatusResponse {
    /// Number of sources queued in the player, including the one playing.
    pub queued: usize,
    pub paused: bool,
    /// Position in the currently playing track, in seconds; absent when
    /// nothing is playing.
    pub position_secs: Option<f64>,
    /// Duration of the currently playing track, in seconds; absent when
    /// unknown or nothing is playing.
    pub duration_secs: Option<f64>,
}

impl From<PlaybackStatus> for PlaybackStatusResponse {
    fn from(status: PlaybackStatus) -> Self {
        PlaybackStatusResponse {
            queued: status.queued,
            paused: status.paused,
            position_secs: status.position.map(|d| d.as_secs_f64()),
            duration_secs: status.duration.map(|d| d.as_secs_f64()),
        }
    }
}

/// A folder in the collection, without its contents.
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiFolderRef {
    pub id: String,
    pub name: String,
    pub kind: String,
}

/// A folder with the folders and tracks directly inside it.
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiFolder {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub folders: Vec<ApiFolderRef>,
    pub tracks: Vec<ApiTrack>,
}

impl ApiFolderRef {
    pub fn from_folder(folder: &Folder) -> Self {
        ApiFolderRef {
            id: folder.id.clone(),
            name: folder.name.clone(),
            kind: kind_name(folder.kind).to_string(),
        }
    }
}

impl ApiFolder {
    pub fn from_folder(folder: &Folder) -> Self {
        ApiFolder {
            id: folder.id.clone(),
            name: folder.name.clone(),
            kind: kind_name(folder.kind).to_string(),
            folders: folder
                .folders
                .values()
                .map(ApiFolderRef::from_folder)
                .collect(),
            tracks: folder.tracks.iter().map(ApiTrack::from_track).collect(),
        }
    }
}

/// The name a folder kind is known by in the API.
pub(crate) fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Category => "category",
        Kind::Artist => "artist",
        Kind::Album => "album",
        Kind::Disc => "disc",
        Kind::Track => "track",
    }
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
    pub categories: Vec<ApiFolderRef>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ApiQueueInput {
    /// The ID of a track, or of any folder (category, artist, album or disc)
    /// to add everything under.
    pub id: String,
    /// Where the added tracks go in the queue. Defaults to the tail.
    #[serde(default)]
    pub position: ApiQueuePosition,
    /// Start playing when the player is idle (the "play this album now"
    /// use case). With `position: replace` the new queue always starts
    /// immediately, since replacing stops the current track.
    #[serde(default)] // Defaults to false
    pub play_now: bool,
}

/// Where newly added tracks go in the queue.
#[derive(Debug, Default, PartialEq, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ApiQueuePosition {
    /// After everything currently queued.
    #[default]
    Tail,
    /// Right after the currently playing track ("play next"). When the
    /// player is idle, they become the next tracks to play.
    Next,
    /// Replace the whole queue with the added tracks. This stops the
    /// currently playing track, if any.
    Replace,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiQueueResponse {
    /// The tracks in the queue, with the track playing right now first.
    pub tracks: Vec<ApiTrack>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ApiQueueRemoveInput {
    /// Index of the track to remove in the queue. Index 0 is the track
    /// playing right now (or the next one to play when the player is idle);
    /// removing it skips to the next track, or stops when it was the last.
    pub index: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ApiQueueMoveInput {
    /// Index of the track to move.
    pub from: usize,
    /// Index to move the track to. While a track is playing, neither `from`
    /// nor `to` may be 0: the front of the queue is the track playing now.
    pub to: usize,
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
