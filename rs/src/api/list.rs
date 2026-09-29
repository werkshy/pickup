use actix_web::{get, web, Responder, Result};

use crate::{
    api::types::{ApiCategory, ListCategoriesResponse},
    app_state::AppState,
};

/// List the categories (top level of the collection).
///
/// TODO fill this out, artists/albums/tracks are not listed yet.
#[utoipa::path(
    tag = "collection",
    responses(
        (status = 200, description = "The categories in the collection", body = ListCategoriesResponse),
    )
)]
#[get("/categories")]
pub async fn list_categories(data: web::Data<AppState>) -> Result<impl Responder> {
    let collection = data.collection.as_ref();

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
