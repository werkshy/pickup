use actix_web::{test, web::Data};

use assert_matches::assert_matches;
use serial_test::serial;

mod helpers;

use helpers::build_test_app_state;
use once_cell::sync::Lazy;
use pickup::{
    app_state::AppState,
    build_app,
    filemanager::{cache::refresh, options::CollectionOptions},
};
use serde_json::{self, json};

// Share one app state (and its single Player thread) across every control test,
// because a fresh Player opens a new audio sink and the volume value we read back
// must persist between the set/get round-trips below.
static APP_STATE: Lazy<Data<AppState>> = Lazy::new(build_test_app_state);

async fn set_volume(value: u8) -> serde_json::Value {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post()
        .uri(format!("/volume/{}", value).as_str())
        .to_request();
    test::call_and_read_body_json(&app, req).await
}

#[serial(queue)]
#[actix_web::test]
async fn test_stop() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post().uri("/stop").to_request();

    let resp = test::call_and_read_body(&app, req).await;

    assert_eq!(&resp[..], b"ok");
}

#[serial(queue)]
#[actix_web::test]
async fn test_play() {
    let options = CollectionOptions {
        dir: String::from("../music"),
        ignores: None,
    };

    let result = refresh(options.clone());

    assert_matches!(result, Ok(_));
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post().uri("/play").to_request();

    let resp = test::call_and_read_body(&app, req).await;

    assert_eq!(&resp[..], b"ok");
}

#[serial(queue)]
#[actix_web::test]
async fn test_set_volume() {
    // The POST handler sets the volume and reads it back, so the response
    // echoes the value we just set.
    let resp = set_volume(50).await;
    assert_eq!(resp, json!({ "volume": 50 }));
}

#[serial(queue)]
#[actix_web::test]
async fn test_get_volume() {
    // Set a known value through the POST endpoint, then read it back through GET.
    let set = set_volume(30).await;
    assert_eq!(set, json!({ "volume": 30 }));

    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::get().uri("/volume").to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp, json!({ "volume": 30 }));
}
