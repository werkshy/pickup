use actix_web::{http::StatusCode, test, web::Data};

mod helpers;

use helpers::build_test_app_state;
use once_cell::sync::Lazy;
use pickup::{app_state::AppState, build_app};
use serde_json::{self, json};

// Build the app state (and load the collection) once for the whole binary.
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

/**
 * A browser navigation (Accept: text/html) to a route nobody serves gets
 * the HTML error page instead of JSON.
 */
#[actix_web::test]
async fn test_unknown_route_with_html_accept_is_html_error() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::get()
        .uri("/does-not-exist")
        .insert_header(("accept", "text/html,application/xhtml+xml"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let body = test::read_body(resp).await;
    let body = String::from_utf8_lossy(&body).to_string();

    assert!(
        content_type.starts_with("text/html"),
        "a browser gets HTML, not JSON: {content_type}"
    );
    assert!(
        body.contains("Page not found"),
        "unexpected error page: {body}"
    );
}
