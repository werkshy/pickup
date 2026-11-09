use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::{get, web, Responder, Result};

use crate::{api::types::ErrorResponse, app_state::AppState, error::AppError};

/**
 * Serve a track's audio file. The browser player loads tracks from here;
 * `NamedFile` serves them with `Range` support, which is what makes seeking
 * work, and picks the content type from the file extension.
 */
#[utoipa::path(
    tag = "tracks",
    params(
        ("id" = String, Path, description = "A track ID"),
    ),
    responses(
        (status = 200, description = "The track's audio file"),
        (status = 206, description = "The byte range requested in the Range header"),
        (status = 404, description = "No matching track found in the collection", body = ErrorResponse),
        (status = 500, description = "The track's file could not be read", body = ErrorResponse),
    )
)]
#[get("/track/{id}")]
pub async fn get(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = path.into_inner();
    let collection = data.collection.get();
    let track = collection
        .get_track(&id)
        .ok_or_else(|| AppError::not_found("No matching music found in the collection"))?;
    let file = PathBuf::from(&collection.dir).join(&track.path);

    NamedFile::open(&file).map_err(|error| {
        log::warn!("Serving {:?} failed: {}", file, error);
        AppError::internal("The track's file could not be read")
    })
}
