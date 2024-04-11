use actix_web::{get, web, Responder, Result};
use lazy_static::__Deref;
use serde::Serialize;

use crate::app_state::AppState;

// TODO fill this out
#[derive(Debug, Serialize)]
struct ApiCategory {
    name: String,
}

#[derive(Debug, Serialize)]
pub struct ApiTrack {
    id: String,
    title: String,
    artist: Option<String>,
    album: Option<String>,
    disc: Option<String>,
    category: String,
}

impl ApiTrack {
    pub fn from_track(track: &crate::filemanager::model::Track) -> Self {
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

#[derive(Debug, Serialize)]
struct ListCategoriesResponse {
    categories: Vec<ApiCategory>,
}

#[get("/categories")]
pub async fn list_categories(data: web::Data<AppState>) -> Result<impl Responder> {
    let collection = data.collection.deref();

    let api_categories: Vec<ApiCategory> = collection
        .values()
        .map(|category| ApiCategory {
            name: category.name.clone(),
        })
        .collect();

    let response = ListCategoriesResponse {
        categories: api_categories,
    };
    Ok(web::Json(response))
}
