use actix_web::{get, web, Responder};

use crate::{
    api::types::{ApiFolder, ApiFolderRef, ErrorResponse, ListCategoriesResponse},
    app_state::AppState,
    error::AppError,
};

/// List the categories (top level of the collection).
#[utoipa::path(
    tag = "collection",
    responses(
        (status = 200, description = "The categories in the collection", body = ListCategoriesResponse),
    )
)]
#[get("/categories")]
pub async fn list_categories(data: web::Data<AppState>) -> impl Responder {
    let collection = data.collection.as_ref();

    let response = ListCategoriesResponse {
        categories: collection.values().map(ApiFolderRef::from_folder).collect(),
    };
    web::Json(response)
}

/// Get one folder by ID, with the folders and tracks directly inside it. Walk
/// the collection by fetching a category from `/categories`, then each child
/// folder from here.
#[utoipa::path(
    tag = "collection",
    responses(
        (status = 200, description = "The folder and its immediate contents", body = ApiFolder),
        (status = 404, description = "No folder with this ID", body = ErrorResponse),
    )
)]
#[get("/folders/{id}")]
pub async fn get_folder(
    data: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = id.into_inner();
    let folder = data
        .collection
        .get_folder(&id)
        .ok_or_else(|| AppError::not_found(format!("No folder found with ID '{id}'")))?;
    Ok(web::Json(ApiFolder::from_folder(folder)))
}
