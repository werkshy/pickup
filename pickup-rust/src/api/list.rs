use actix_web::{get, web, Responder, Result};
use lazy_static::__Deref;
use serde::Serialize;

use crate::app_state::AppState;

#[derive(Debug, Serialize)]
struct ApiCategory {
    name: String,
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
