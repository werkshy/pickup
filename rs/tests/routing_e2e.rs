use actix_web::{http::StatusCode, test, web::Data};

mod helpers;

use helpers::build_test_app_state;
use once_cell::sync::Lazy;
use pickup::{app_state::AppState, build_app};
use serde_json::{self, json};

// Build (and open the collection DB) once for the whole binary, so the tests
// don't race each other opening sled.
static APP_STATE: Lazy<Data<AppState>> = Lazy::new(build_test_app_state);

#[actix_web::test]
async fn test_unknown_route_is_404() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::get().uri("/does-not-exist").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body,
        json!({ "code": "not_found", "message": "No such route" })
    );
}

#[actix_web::test]
async fn test_method_mismatch_on_root_is_404() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post().uri("/").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body,
        json!({ "code": "not_found", "message": "No such route" })
    );
}

#[actix_web::test]
async fn test_method_mismatch_on_openapi_is_404() {
    // /openapi.json is registered outside register_services, via
    // openapi_service, so it needs its own fallback.
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post().uri("/openapi.json").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body,
        json!({ "code": "not_found", "message": "No such route" })
    );
}

#[actix_web::test]
async fn test_method_mismatch_on_dynamic_path_is_404() {
    // The path pattern matches even though the method does not, so this must
    // still come back in the standard error shape.
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::get().uri("/volume/300").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body,
        json!({ "code": "not_found", "message": "No such route" })
    );
}
