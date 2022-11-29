use actix_web::test;

mod helpers;

use helpers::build_test_app;
use serde_json::{self, json};

#[actix_web::test]
async fn test_list_categories() {
    let app = test::init_service(build_test_app()).await;
    let req = test::TestRequest::get().uri("/categories").to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(
        resp,
        json!({ "categories": [{"name": "Music"}, {"name": "Free"}] })
    );
}
