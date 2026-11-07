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

    assert_eq!(resp.get("tracks").unwrap().as_array().unwrap().len(), 0);
}

async fn get_json(uri: &str) -> serde_json::Value {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::get().uri(uri).to_request();
    test::call_and_read_body_json(&app, req).await
}

async fn add_to_queue(body: serde_json::Value) -> serde_json::Value {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post()
        .uri("/queue/add")
        .set_json(body)
        .to_request();
    test::call_and_read_body_json(&app, req).await
}

async fn play_track(id: String) {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post()
        .uri(format!("/play/{id}").as_str())
        .to_request();
    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp, json!({ "status": "ok" }));
}

const TRACK_1: &str = "Bryan Teoh/Free/01 - Finally See The Light.mp3";
const TRACK_2: &str = "Bryan Teoh/Free/02 - Neptunian Princess.mp3";

/// Playing a track and then adding another with position "next" must leave
/// the new track queued directly behind the playing one, without starting
/// it early.
#[serial(queue)]
#[actix_web::test]
async fn test_add_next_goes_after_the_current_track() {
    clear_playlist().await;
    play_track(generate_track_id(Path::new(TRACK_1))).await;

    let resp = add_to_queue(json!({
        "id": generate_track_id(Path::new(TRACK_2)),
        "position": "next",
    }))
    .await;

    let tracks = resp.get("tracks").unwrap().as_array().unwrap();
    assert_eq!(tracks.len(), 2);
    assert_eq!(tracks[0]["id"], generate_track_id(Path::new(TRACK_1)));
    assert_eq!(tracks[1]["id"], generate_track_id(Path::new(TRACK_2)));

    // The next track must not start while one is playing.
    let status = get_json("/status").await;
    assert_eq!(status["queued"], 1);

    clear_playlist().await;
}

/// play_now must start playback when the player is idle.
#[serial(queue)]
#[actix_web::test]
async fn test_add_with_play_now_starts_playback() {
    clear_playlist().await;

    add_to_queue(json!({
        "id": generate_track_id(Path::new(TRACK_2)),
        "play_now": true,
    }))
    .await;

    let status = get_json("/status").await;
    assert_eq!(status["queued"], 1);

    clear_playlist().await;
}

/// play_now must not interrupt playback that is already running.
#[serial(queue)]
#[actix_web::test]
async fn test_add_with_play_now_does_not_interrupt_playback() {
    clear_playlist().await;
    play_track(generate_track_id(Path::new(TRACK_1))).await;

    add_to_queue(json!({
        "id": generate_track_id(Path::new(TRACK_2)),
        "play_now": true,
    }))
    .await;

    // Still only the first track is playing.
    let status = get_json("/status").await;
    assert_eq!(status["queued"], 1);
    let queue = get_json("/queue").await;
    assert_eq!(queue["tracks"].as_array().unwrap().len(), 2);

    clear_playlist().await;
}

/// position "replace" swaps the queue. With play_now the new queue starts
/// immediately (replacing also stops the old track); without it, playback
/// stops.
#[serial(queue)]
#[actix_web::test]
async fn test_add_replace_with_play_now_starts_the_new_queue() {
    clear_playlist().await;
    play_track(generate_track_id(Path::new(TRACK_1))).await;

    let resp = add_to_queue(json!({
        "id": generate_track_id(Path::new(TRACK_2)),
        "position": "replace",
        "play_now": true,
    }))
    .await;

    let tracks = resp.get("tracks").unwrap().as_array().unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0]["id"], generate_track_id(Path::new(TRACK_2)));

    let status = get_json("/status").await;
    assert_eq!(status["queued"], 1);

    clear_playlist().await;
}

#[serial(queue)]
#[actix_web::test]
async fn test_add_replace_without_play_now_stops_playback() {
    clear_playlist().await;
    play_track(generate_track_id(Path::new(TRACK_1))).await;

    add_to_queue(json!({
        "id": generate_track_id(Path::new(TRACK_2)),
        "position": "replace",
    }))
    .await;

    let status = get_json("/status").await;
    assert_eq!(status["queued"], 0);
    let queue = get_json("/queue").await;
    assert_eq!(queue["tracks"].as_array().unwrap().len(), 1);

    clear_playlist().await;
}

/// Clearing the queue must also stop playback: the playing track would no
/// longer be on the queue.
#[serial(queue)]
#[actix_web::test]
async fn test_clear_stops_playback() {
    clear_playlist().await;
    play_track(generate_track_id(Path::new(TRACK_1))).await;

    clear_playlist().await;

    let status = get_json("/status").await;
    assert_eq!(status["queued"], 0);
}

/// Removing a queued (not playing) track leaves playback untouched.
#[serial(queue)]
#[actix_web::test]
async fn test_remove_removes_a_queued_track() {
    clear_playlist().await;
    play_track(generate_track_id(Path::new(TRACK_1))).await;
    add_to_queue(json!({ "id": generate_track_id(Path::new(TRACK_2)) })).await;

    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post()
        .uri("/queue/remove")
        .set_json(json!({ "index": 1 }))
        .to_request();
    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    let tracks = resp.get("tracks").unwrap().as_array().unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0]["id"], generate_track_id(Path::new(TRACK_1)));

    // The playing track was not touched.
    let status = get_json("/status").await;
    assert_eq!(status["queued"], 1);

    clear_playlist().await;
}

/// Removing the track that is playing (index 0) skips to the next one.
#[serial(queue)]
#[actix_web::test]
async fn test_remove_playing_track_skips_to_next() {
    clear_playlist().await;
    play_track(generate_track_id(Path::new(TRACK_1))).await;
    add_to_queue(json!({ "id": generate_track_id(Path::new(TRACK_2)) })).await;

    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post()
        .uri("/queue/remove")
        .set_json(json!({ "index": 0 }))
        .to_request();
    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    let tracks = resp.get("tracks").unwrap().as_array().unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0]["id"], generate_track_id(Path::new(TRACK_2)));

    // The next track is now playing.
    let status = get_json("/status").await;
    assert_eq!(status["queued"], 1);

    clear_playlist().await;
}

/// Removing the only playing track stops playback and empties the queue.
#[serial(queue)]
#[actix_web::test]
async fn test_remove_last_playing_track_stops() {
    clear_playlist().await;
    play_track(generate_track_id(Path::new(TRACK_1))).await;

    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post()
        .uri("/queue/remove")
        .set_json(json!({ "index": 0 }))
        .to_request();
    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.get("tracks").unwrap().as_array().unwrap().len(), 0);

    let status = get_json("/status").await;
    assert_eq!(status["queued"], 0);
    assert_eq!(status["paused"], false);
}

#[serial(queue)]
#[actix_web::test]
async fn test_remove_out_of_range_is_bad_request() {
    clear_playlist().await;
    add_to_queue(json!({ "id": generate_track_id(Path::new(TRACK_1)) })).await;

    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post()
        .uri("/queue/remove")
        .set_json(json!({ "index": 5 }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["code"], "invalid_request");

    clear_playlist().await;
}

/// Reordering an idle queue must be possible in any direction, including
/// position 0 (the next track to play).
#[serial(queue)]
#[actix_web::test]
async fn test_move_reorders_the_queue() {
    clear_playlist().await;
    add_to_queue(json!({ "id": generate_track_id(Path::new(TRACK_1)) })).await;
    add_to_queue(json!({ "id": generate_track_id(Path::new(TRACK_2)) })).await;

    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post()
        .uri("/queue/move")
        .set_json(json!({ "from": 0, "to": 1 }))
        .to_request();
    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    let tracks = resp.get("tracks").unwrap().as_array().unwrap();
    assert_eq!(tracks.len(), 2);
    assert_eq!(tracks[0]["id"], generate_track_id(Path::new(TRACK_2)));
    assert_eq!(tracks[1]["id"], generate_track_id(Path::new(TRACK_1)));

    clear_playlist().await;
}

/// While a track is playing, position 0 is that track: neither it nor
/// anything moved onto it may be moved.
#[serial(queue)]
#[actix_web::test]
async fn test_move_playing_track_is_rejected() {
    clear_playlist().await;
    play_track(generate_track_id(Path::new(TRACK_1))).await;
    add_to_queue(json!({ "id": generate_track_id(Path::new(TRACK_2)) })).await;

    let app = test::init_service(build_app((*APP_STATE).clone())).await;

    for body in [json!({ "from": 0, "to": 1 }), json!({ "from": 1, "to": 0 })] {
        let req = test::TestRequest::post()
            .uri("/queue/move")
            .set_json(body)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
        let body: serde_json::Value = read_body_json(resp).await;
        assert_eq!(body["code"], "invalid_request");
    }

    // The queue is unchanged.
    let queue = get_json("/queue").await;
    let tracks = queue.get("tracks").unwrap().as_array().unwrap();
    assert_eq!(tracks[0]["id"], generate_track_id(Path::new(TRACK_1)));
    assert_eq!(tracks[1]["id"], generate_track_id(Path::new(TRACK_2)));

    clear_playlist().await;
}

#[serial(queue)]
#[actix_web::test]
async fn test_move_out_of_range_is_bad_request() {
    clear_playlist().await;
    add_to_queue(json!({ "id": generate_track_id(Path::new(TRACK_1)) })).await;

    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post()
        .uri("/queue/move")
        .set_json(json!({ "from": 0, "to": 5 }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["code"], "invalid_request");

    clear_playlist().await;
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

    assert_eq!(resp.get("tracks").unwrap().as_array().unwrap().len(), 5);

    let req = test::TestRequest::post().uri("/queue/clear").to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.get("tracks").unwrap().as_array().unwrap().len(), 0);
}
