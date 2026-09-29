use actix_web::test;

mod helpers;

use helpers::build_test_app;

/**
 * The spec must be reachable and must document every route we serve, so new
 * endpoints don't silently go undocumented.
 */
#[actix_web::test]
async fn test_openapi_lists_all_routes() {
    let app = test::init_service(build_test_app()).await;
    let req = test::TestRequest::get().uri("/openapi.json").to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    let paths = resp["paths"].as_object().unwrap();
    for route in [
        "/",
        "/play",
        "/stop",
        "/volume",
        "/volume/{volume}",
        "/categories",
        "/queue",
        "/queue/add",
        "/queue/clear",
    ] {
        assert!(
            paths.contains_key(route),
            "route {route} is missing from the generated spec"
        );
    }
}
