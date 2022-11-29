use actix_web::{get, web, Responder, Result};
use itertools::sorted;
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
    let mut api_categories: Vec<ApiCategory> = vec![];

    sorted(collection.keys()).for_each(|category| {
        api_categories.push(ApiCategory {
            name: collection.get(category).unwrap().name.clone(),
        })
    });
    let response = ListCategoriesResponse {
        categories: api_categories,
    };
    Ok(web::Json(response))
}
