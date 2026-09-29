use actix_web::test;

use serde_json::json;

use crate::helpers::build_test_app;

mod helpers;

#[actix_web::test]
async fn test_index_get() {
    let app = test::init_service(build_test_app()).await;
    let req = test::TestRequest::get().uri("/").to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(
        resp,
        json!({ "name": "pickup", "version": env!("CARGO_PKG_VERSION") })
    );
}
