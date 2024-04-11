use actix_web::{
    test::{self, read_body_json},
    web::Data,
};

use serial_test::serial;

mod helpers;

use helpers::build_test_app_state;
use once_cell::sync::Lazy;
use pickup::{app_state::AppState, build_app};
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
        .set_json(json!({"category": "DoesNotExist"}))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    let resp_json: serde_json::Value = read_body_json(resp).await;
    assert_eq!(
        resp_json,
        json!({ "error": "No matching music found in the collection" })
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
            "category": "Music",
        }))
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.get("position").unwrap().as_u64().unwrap(), 0);
    assert_eq!(resp.get("tracks").unwrap().as_array().unwrap().len(), 5);

    // Now if we re-add the category, we should get 5 more tracks
    let req = test::TestRequest::post()
        .uri("/queue/add")
        .set_json(json!({
            "category": "Music",
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
            "category": "Music",
            "artist": "Alex-Productions",
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
            "category": "Music",
            "artist": "Bryan Teoh",
            "album": "Free",
        }))
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.get("position").unwrap().as_u64().unwrap(), 0);
    assert_eq!(resp.get("tracks").unwrap().as_array().unwrap().len(), 2);
}

#[serial(queue)]
#[actix_web::test]
async fn test_add_track() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    clear_playlist().await;
    let req = test::TestRequest::post()
        .uri("/queue/add")
        .set_json(json!({
            "category": "Music",
            "artist": "Bryan Teoh",
            "album": "Free",
            "track": "01 - Finally See The Light",
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
            "category": "Music",
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
