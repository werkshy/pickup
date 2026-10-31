use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use actix_web::{http::StatusCode, test, web::Data};

use assert_matches::assert_matches;
use serial_test::serial;

mod helpers;

use helpers::build_test_app_state;
use once_cell::sync::Lazy;
use pickup::{
    app_state::AppState,
    build_app,
    filemanager::{
        cache::refresh,
        collection::Collection,
        model::{CollectionLocation, Kind, Track},
        options::CollectionOptions,
        utils::generate_track_id,
    },
    queue::PlaybackQueue,
    spawn_player,
};
use serde_json::{self, json};

// Share one app state (and its single Player thread) across every control test,
// because a fresh Player opens a new audio sink and the volume value we read back
// must persist between the set/get round-trips below.
static APP_STATE: Lazy<Data<AppState>> = Lazy::new(build_test_app_state);

async fn set_volume(value: u32) -> serde_json::Value {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post()
        .uri(format!("/volume/{}", value).as_str())
        .to_request();
    test::call_and_read_body_json(&app, req).await
}

async fn set_stop() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post().uri("/stop").to_request();
    let _: serde_json::Value = test::call_and_read_body_json(&app, req).await;
}

#[serial(queue)]
#[actix_web::test]
async fn test_status_reports_empty_player() {
    // Reset to a known state: stop clears everything queued in the player,
    // regardless of what earlier tests in this serial group played.
    set_stop().await;

    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::get().uri("/status").to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(
        resp,
        json!({
            "queued": 0,
            "paused": false,
            "position_secs": null,
            "duration_secs": null,
        })
    );
}

#[serial(queue)]
#[actix_web::test]
async fn test_status_reports_playing_track() {
    let options = CollectionOptions {
        dir: String::from("../music"),
        ignores: None,
    };
    assert_matches!(refresh(options), Ok(_));

    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let id = generate_track_id(Path::new("Bryan Teoh/Free/01 - Finally See The Light.mp3"));
    let req = test::TestRequest::post()
        .uri(format!("/play/{id}").as_str())
        .to_request();
    let _: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    let req = test::TestRequest::get().uri("/status").to_request();
    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp["queued"], 1);
    assert_eq!(resp["paused"], false);
    // The decoder determined the duration of the track, and playback has a
    // position.
    assert!(
        resp["duration_secs"].as_f64().is_some_and(|d| d > 0.0),
        "unexpected duration in {resp}"
    );
    assert!(
        resp["position_secs"].as_f64().is_some_and(|p| p >= 0.0),
        "unexpected position in {resp}"
    );

    // Leave the player empty for the other tests in this serial group.
    set_stop().await;
}

#[serial(queue)]
#[actix_web::test]
async fn test_play_resumes_from_the_queue() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;

    // Start from a cleared queue, then fill it without starting playback.
    let req = test::TestRequest::post().uri("/queue/clear").to_request();
    let _: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    let req = test::TestRequest::post()
        .uri("/queue/add")
        .set_json(serde_json::json!({
            "id": generate_track_id(Path::new("Bryan Teoh/Free/01 - Finally See The Light.mp3")),
        }))
        .to_request();
    let _: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    let req = test::TestRequest::post()
        .uri("/queue/add")
        .set_json(serde_json::json!({
            "id": generate_track_id(Path::new("Bryan Teoh/Free/02 - Neptunian Princess.mp3")),
        }))
        .to_request();
    let _: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    let req = test::TestRequest::get().uri("/status").to_request();
    let status: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(status["queued"], 0);

    // Bare /play starts the track at the front of the queue.
    let req = test::TestRequest::post().uri("/play").to_request();
    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp, serde_json::json!({ "status": "ok" }));

    let req = test::TestRequest::get().uri("/status").to_request();
    let status: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(status["queued"], 1);

    // Stopping keeps the stopped track at the front, so /play resumes it.
    set_stop().await;
    let req = test::TestRequest::post().uri("/play").to_request();
    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp, serde_json::json!({ "status": "ok" }));

    let req = test::TestRequest::get().uri("/status").to_request();
    let status: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(status["queued"], 1);

    set_stop().await;
}

#[serial(queue)]
#[actix_web::test]
async fn test_play_with_exhausted_queue_is_bad_request() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post().uri("/queue/clear").to_request();
    let _: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    let resp = test::call_service(&app, test::TestRequest::post().uri("/play").to_request()).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], "invalid_request");
}

#[serial(queue)]
#[actix_web::test]
async fn test_play_replaces_the_current_track() {
    let options = CollectionOptions {
        dir: String::from("../music"),
        ignores: None,
    };
    assert_matches!(refresh(options), Ok(_));
    set_stop().await;

    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let first = generate_track_id(Path::new("Bryan Teoh/Free/01 - Finally See The Light.mp3"));
    let second = generate_track_id(Path::new("Bryan Teoh/Free/02 - Neptunian Princess.mp3"));

    let req = test::TestRequest::post()
        .uri(format!("/play/{first}").as_str())
        .to_request();
    let _: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    let req = test::TestRequest::get().uri("/status").to_request();
    let playing: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(playing["queued"], 1);

    // Playing another track must replace the first, not queue behind it.
    let req = test::TestRequest::post()
        .uri(format!("/play/{second}").as_str())
        .to_request();
    let _: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    let req = test::TestRequest::get().uri("/status").to_request();
    let replaced: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(replaced["queued"], 1);
    assert_ne!(
        replaced["duration_secs"], playing["duration_secs"],
        "duration still describes the replaced track: {replaced}"
    );
    assert!(
        replaced["position_secs"].as_f64().unwrap() < 1.0,
        "the replaced track did not start from the beginning: {replaced}"
    );

    // The queue was replaced with the single new track as well.
    let req = test::TestRequest::get().uri("/queue").to_request();
    let queue: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(queue["tracks"].as_array().unwrap().len(), 1);
    assert_eq!(queue["tracks"][0]["id"], second);

    set_stop().await;
}

#[serial(queue)]
#[actix_web::test]
async fn test_stop() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post().uri("/stop").to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp, json!({ "status": "ok" }));
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
    let id = generate_track_id(Path::new("Bryan Teoh/Free/01 - Finally See The Light.mp3"));
    let req = test::TestRequest::post()
        .uri(format!("/play/{id}").as_str())
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp, json!({ "status": "ok" }));
}

#[serial(queue)]
#[actix_web::test]
async fn test_play_with_unknown_track_id_is_404() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post().uri("/play/trk_nope").to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body,
        json!({ "code": "not_found", "message": "No track found with ID 'trk_nope'" })
    );
}

/**
 * When the track's file cannot be opened, the play endpoint must report the
 * underlying file error instead of pretending the playback started, and it
 * must leave the current playback and queue untouched.
 */
#[serial(queue)]
#[actix_web::test]
async fn test_play_with_missing_file_returns_error() {
    // Build an app state with a real, playable track and one that points at
    // a file that does not exist on disk.
    let missing_file = PathBuf::from("definitely/missing/file.wav");
    let missing_location = CollectionLocation {
        category: String::from("test"),
        artist: Some(String::from("definitely")),
        album: Some(String::from("missing")),
        disc: None,
    };
    let missing_track = Track {
        id: String::from("missing"),
        name: String::from("missing track"),
        extension: String::from("wav"),
        path: missing_file,
        category: missing_location.category.clone(),
        artist: missing_location.artist.clone(),
        album: missing_location.album.clone(),
        disc: None,
    };

    let real_location = CollectionLocation {
        category: String::from("Music"),
        artist: Some(String::from("Bryan Teoh")),
        album: Some(String::from("Free")),
        disc: None,
    };
    let real_track = Track {
        id: String::from("real"),
        name: String::from("01 - Finally See The Light"),
        extension: String::from("mp3"),
        path: PathBuf::from("Bryan Teoh/Free/01 - Finally See The Light.mp3"),
        category: real_location.category.clone(),
        artist: real_location.artist.clone(),
        album: real_location.album.clone(),
        disc: None,
    };

    let mut collection = Collection::new(String::from("../music"));
    collection.add_folder(
        Kind::Album,
        &missing_location,
        Path::new("definitely/missing"),
    );
    collection.add_track(missing_track, &missing_location);
    collection.add_folder(Kind::Album, &real_location, Path::new("Bryan Teoh/Free"));
    collection.add_track(real_track, &real_location);

    // The queue Arc must be the same one the Player thread holds, or the
    // Player would play from a different queue than the API mutates.
    let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
    let app_state = Data::new(AppState {
        player: spawn_player(queue.clone(), PathBuf::from("../music")),
        collection: Arc::new(collection),
        queue,
    });

    let app = test::init_service(build_app(app_state)).await;

    // Start playing the real track.
    let req = test::TestRequest::post().uri("/play/real").to_request();
    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp, json!({ "status": "ok" }));

    // Playing a broken track fails without disturbing the current playback.
    let req = test::TestRequest::post().uri("/play/missing").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], "playback_failed");
    let message = body["message"].as_str().unwrap();
    assert!(
        message.contains("could not open ../music/definitely/missing/file.wav"),
        "unexpected error body: {message}"
    );

    let req = test::TestRequest::get().uri("/status").to_request();
    let status: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(
        status["queued"], 1,
        "the real track should still be playing"
    );

    let req = test::TestRequest::get().uri("/queue").to_request();
    let queue: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(queue["tracks"].as_array().unwrap().len(), 1);
    assert_eq!(queue["tracks"][0]["id"], "real");

    let req = test::TestRequest::post().uri("/stop").to_request();
    let _: serde_json::Value = test::call_and_read_body_json(&app, req).await;
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

#[serial(queue)]
#[actix_web::test]
async fn test_set_volume_clamps_out_of_range() {
    // Values over 100 are clamped by the handler rather than rejected or
    // mistaken for an unknown route.
    let resp = set_volume(300).await;
    assert_eq!(resp, json!({ "volume": 100 }));
}

#[serial(queue)]
#[actix_web::test]
async fn test_set_volume_rejects_non_numeric() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post().uri("/volume/abc").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["code"], "invalid_request");
}
