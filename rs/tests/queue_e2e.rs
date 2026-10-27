use std::path::Path;

use actix_web::{
    test::{self, read_body_json},
    web::Data,
};

use serial_test::serial;

mod helpers;

use helpers::build_test_app_state;
use once_cell::sync::Lazy;
use pickup::{
    app_state::AppState,
    build_app,
    filemanager::utils::{
        generate_album_id, generate_artist_id, generate_category_id, generate_disc_id,
        generate_track_id,
    },
};
use serde_json::{self, json};

// Make a shared app state for use in all tests, because Sled won't let us open multiple instances of the DB file
// from differet threads.
static APP_STATE: Lazy<Data<AppState>> = Lazy::new(build_test_app_state);

pub async fn clear_playlist() {
    let service = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post().uri("/queue/clear").to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&service, req).await;

    assert_eq!(resp.get("position").unwrap().as_u64().unwrap(), 0);
    assert_eq!(resp.get("tracks").unwrap().as_array().unwrap().len(), 0);
}

#[serial(queue)]
#[actix_web::test]
async fn test_not_found() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    clear_playlist().await;
    let req = test::TestRequest::post()
        .uri("/queue/add")
        .set_json(json!({"id": "cat_nope"}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    let resp_json: serde_json::Value = read_body_json(resp).await;
    assert_eq!(
        resp_json,
        json!({ "code": "not_found", "message": "No matching music found in the collection" })
    );
}

/**
 * A body that doesn't deserialize is rejected before the handler runs, so
 * these don't need the queue lock.
 */
#[actix_web::test]
async fn test_add_missing_id_is_bad_request() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post()
        .uri("/queue/add")
        .set_json(json!({ "artist": "Bryan Teoh" }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["code"], "invalid_request");
    let message = body["message"].as_str().unwrap();
    // The trailing "at line N column M" varies with the payload, so only the
    // prefix and the offending field are checked.
    assert!(
        message.starts_with("Invalid JSON body: "),
        "unexpected error body: {message}"
    );
    assert!(message.contains("`id`"), "unexpected error body: {message}");
}

#[actix_web::test]
async fn test_add_malformed_json_is_bad_request() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post()
        .uri("/queue/add")
        .insert_header(("content-type", "application/json"))
        .set_payload(r#"{"id": "x""#)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["code"], "invalid_request");
    let message = body["message"].as_str().unwrap();
    assert!(
        message.starts_with("Invalid JSON body: "),
        "unexpected error body: {message}"
    );
}

#[actix_web::test]
async fn test_add_wrong_content_type_is_bad_request() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post()
        .uri("/queue/add")
        .insert_header(("content-type", "text/plain"))
        .set_payload(r#"{"id": "x"}"#)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(
        body,
        json!({
            "code": "invalid_request",
            "message": "Expected a JSON body with Content-Type: application/json",
        })
    );
}

#[serial(queue)]
#[actix_web::test]
async fn test_add_category() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    clear_playlist().await;
    let req = test::TestRequest::post()
        .uri("/queue/add")
        .set_json(json!({
            "id": generate_category_id(Path::new("Music")),
        }))
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.get("position").unwrap().as_u64().unwrap(), 0);
    assert_eq!(resp.get("tracks").unwrap().as_array().unwrap().len(), 5);

    // Now if we re-add the category, we should get 5 more tracks
    let req = test::TestRequest::post()
        .uri("/queue/add")
        .set_json(json!({
            "id": generate_category_id(Path::new("Music")),
        }))
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp.get("tracks").unwrap().as_array().unwrap().len(), 10);
}

#[serial(queue)]
#[actix_web::test]
async fn test_add_artist() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    clear_playlist().await;
    let req = test::TestRequest::post()
        .uri("/queue/add")
        .set_json(json!({
            "id": generate_artist_id(Path::new("Alex-Productions")),
        }))
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.get("error"), None);
    assert_eq!(resp.get("position").unwrap().as_u64().unwrap(), 0);
    assert_eq!(resp.get("tracks").unwrap().as_array().unwrap().len(), 1);
}

#[serial(queue)]
#[actix_web::test]
async fn test_add_album() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    clear_playlist().await;
    let req = test::TestRequest::post()
        .uri("/queue/add")
        .set_json(json!({
            "id": generate_album_id(Path::new("Bryan Teoh/Free")),
        }))
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.get("position").unwrap().as_u64().unwrap(), 0);
    assert_eq!(resp.get("tracks").unwrap().as_array().unwrap().len(), 2);
}

#[serial(queue)]
#[actix_web::test]
async fn test_add_disc() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    clear_playlist().await;
    let req = test::TestRequest::post()
        .uri("/queue/add")
        .set_json(json!({
            "id": generate_disc_id(Path::new("_Free/Various Artists/Microsong Challenge/Disc 1")),
        }))
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.get("position").unwrap().as_u64().unwrap(), 0);
    assert_eq!(resp.get("tracks").unwrap().as_array().unwrap().len(), 14);
}

#[serial(queue)]
#[actix_web::test]
async fn test_add_track() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    clear_playlist().await;
    let req = test::TestRequest::post()
        .uri("/queue/add")
        .set_json(json!({
            "id": generate_track_id(Path::new(
                "Bryan Teoh/Free/01 - Finally See The Light.mp3",
            )),
        }))
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.get("position").unwrap().as_u64().unwrap(), 0);
    assert_eq!(resp.get("tracks").unwrap().as_array().unwrap().len(), 1);
}

#[serial(queue)]
#[actix_web::test]
async fn test_clear() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    clear_playlist().await;
    let req = test::TestRequest::post()
        .uri("/queue/add")
        .set_json(json!({
            "id": generate_category_id(Path::new("Music")),
        }))
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.get("position").unwrap().as_u64().unwrap(), 0);
    assert_eq!(resp.get("tracks").unwrap().as_array().unwrap().len(), 5);

    let req = test::TestRequest::post().uri("/queue/clear").to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.get("position").unwrap().as_u64().unwrap(), 0);
    assert_eq!(resp.get("tracks").unwrap().as_array().unwrap().len(), 0);
}
