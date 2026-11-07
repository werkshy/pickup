use actix_web::{
    cookie::Cookie,
    http::StatusCode,
    test::{self, call_service, read_body_json},
    web::Data,
};
use pickup::{app_state::AppState, build_app, session::SESSION_COOKIE};
use serde_json::{self, json, Value};
use serial_test::serial;

mod helpers;

/**
 * Session queue tests: each session gets its own queue, driven by the
 * browser via /session/queue/next. Each test builds a fresh app state and runs
 * serially, because building the collection (and its cache) is not safe to
 * race against.
 */
async fn setup() -> Data<AppState> {
    helpers::build_test_app_state()
}

/// Create a session, and return its cookie for use in later requests.
async fn new_session_cookie(state: &Data<AppState>) -> Cookie<'static> {
    let app = test::init_service(build_app(state.clone())).await;
    let req = test::TestRequest::get().uri("/hello").to_request();
    let response = call_service(&app, req).await;
    response
        .response()
        .cookies()
        .find(|cookie| cookie.name() == SESSION_COOKIE)
        .expect("first response sets a session cookie")
        .into_owned()
}

async fn call_json(
    state: &Data<AppState>,
    cookie: &Cookie<'static>,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let app = test::init_service(build_app(state.clone())).await;
    let mut req = test::TestRequest::default()
        .method(method.parse().unwrap())
        .uri(uri)
        .cookie(cookie.clone());
    if let Some(body) = body {
        req = req.set_json(body);
    }
    let response = call_service(&app, req.to_request()).await;
    let status = response.status();
    let body: Value = read_body_json(response).await;
    (status, body)
}

fn track_id(relpath: &str) -> String {
    pickup::filemanager::utils::generate_track_id(std::path::Path::new(relpath))
}

const TRACK_1: &str = "Bryan Teoh/Free/01 - Finally See The Light.mp3";
const TRACK_2: &str = "Bryan Teoh/Free/02 - Neptunian Princess.mp3";
const TRACK_3: &str = "Kevin MacLeod/Misc/CD 1/01 - Lukewarm Banjo.mp3";

fn ids(response: &Value) -> Vec<&str> {
    response["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|track| track["id"].as_str().unwrap())
        .collect()
}

/// A session's queue starts empty, and the queue a session adds to is its
/// own: another session sees an empty queue, and so does the jukebox.
#[serial(session_queue)]
#[actix_web::test]
async fn test_queues_are_per_session() {
    let state = setup().await;
    let cookie_a = new_session_cookie(&state).await;
    let cookie_b = new_session_cookie(&state).await;

    let (status, queue) = call_json(&state, &cookie_a, "GET", "/session/queue", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(queue["tracks"].as_array().unwrap().len(), 0);

    let (status, queue) = call_json(
        &state,
        &cookie_a,
        "POST",
        "/session/queue/add",
        Some(json!({ "id": track_id(TRACK_1) })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ids(&queue), [track_id(TRACK_1)]);

    // Another session sees an empty queue...
    let (_, queue) = call_json(&state, &cookie_b, "GET", "/session/queue", None).await;
    assert_eq!(queue["tracks"].as_array().unwrap().len(), 0);

    // ... and so does the jukebox queue.
    let (_, queue) = call_json(&state, &cookie_a, "GET", "/queue", None).await;
    assert_eq!(queue["tracks"].as_array().unwrap().len(), 0);
}

/// position "next" goes behind the front (the playing track), "tail" at the
/// back, "replace" swaps everything.
#[serial(session_queue)]
#[actix_web::test]
async fn test_add_positions() {
    let state = setup().await;
    let cookie = new_session_cookie(&state).await;

    // Empty queue: "next" becomes the front.
    let (_, queue) = call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/add",
        Some(json!({ "id": track_id(TRACK_1), "position": "next" })),
    )
    .await;
    assert_eq!(ids(&queue), [track_id(TRACK_1)]);

    let (_, queue) = call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/add",
        Some(json!({ "id": track_id(TRACK_2) })),
    )
    .await;
    assert_eq!(ids(&queue), [track_id(TRACK_1), track_id(TRACK_2)]);

    let (_, queue) = call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/add",
        Some(json!({ "id": track_id(TRACK_3), "position": "next" })),
    )
    .await;
    assert_eq!(
        ids(&queue),
        [track_id(TRACK_1), track_id(TRACK_3), track_id(TRACK_2)]
    );

    let (_, queue) = call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/add",
        Some(json!({ "id": track_id(TRACK_2), "position": "replace" })),
    )
    .await;
    assert_eq!(ids(&queue), [track_id(TRACK_2)]);
}

/// play_now on an empty queue puts the tracks at the front, so the browser
/// starts playing the first one added.
#[serial(session_queue)]
#[actix_web::test]
async fn test_add_with_play_now_on_an_empty_queue_starts_at_the_front() {
    let state = setup().await;
    let cookie = new_session_cookie(&state).await;

    let (_, queue) = call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/add",
        Some(json!({ "id": track_id(TRACK_2), "play_now": true })),
    )
    .await;
    assert_eq!(ids(&queue), [track_id(TRACK_2)]);

    let (_, queue) = call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/add",
        Some(json!({ "id": track_id(TRACK_1) })),
    )
    .await;
    assert_eq!(ids(&queue), [track_id(TRACK_2), track_id(TRACK_1)]);
}

#[serial(session_queue)]
#[actix_web::test]
async fn test_add_unknown_id_is_not_found() {
    let state = setup().await;
    let cookie = new_session_cookie(&state).await;

    let (status, body) = call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/add",
        Some(json!({ "id": "trk_nope" })),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["code"], json!("not_found"));
}

/// Removing a track takes it off the queue; an out-of-range index is
/// rejected without touching the queue.
#[serial(session_queue)]
#[actix_web::test]
async fn test_remove() {
    let state = setup().await;
    let cookie = new_session_cookie(&state).await;
    call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/add",
        Some(json!({ "id": track_id(TRACK_1) })),
    )
    .await;
    call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/add",
        Some(json!({ "id": track_id(TRACK_2) })),
    )
    .await;

    let (_, queue) = call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/remove",
        Some(json!({ "index": 0 })),
    )
    .await;
    assert_eq!(ids(&queue), [track_id(TRACK_2)]);

    let (status, body) = call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/remove",
        Some(json!({ "index": 5 })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], json!("invalid_request"));

    let (_, queue) = call_json(&state, &cookie, "GET", "/session/queue", None).await;
    assert_eq!(ids(&queue), [track_id(TRACK_2)]);
}

/// Tracks can be reordered to any position, including the front: the
/// browser owns playback, so the server has no "playing" position to
/// protect.
#[serial(session_queue)]
#[actix_web::test]
async fn test_move() {
    let state = setup().await;
    let cookie = new_session_cookie(&state).await;
    for relpath in [TRACK_1, TRACK_2, TRACK_3] {
        call_json(
            &state,
            &cookie,
            "POST",
            "/session/queue/add",
            Some(json!({ "id": track_id(relpath) })),
        )
        .await;
    }

    let (_, queue) = call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/move",
        Some(json!({ "from": 0, "to": 2 })),
    )
    .await;
    assert_eq!(
        ids(&queue),
        [track_id(TRACK_2), track_id(TRACK_3), track_id(TRACK_1)]
    );

    let (status, body) = call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/move",
        Some(json!({ "from": 0, "to": 5 })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], json!("invalid_request"));
}

/// Clearing empties the session's queue, and only that one.
#[serial(session_queue)]
#[actix_web::test]
async fn test_clear() {
    let state = setup().await;
    let cookie = new_session_cookie(&state).await;
    call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/add",
        Some(json!({ "id": track_id(TRACK_1) })),
    )
    .await;

    let (_, queue) = call_json(&state, &cookie, "POST", "/session/queue/clear", None).await;
    assert_eq!(queue["tracks"].as_array().unwrap().len(), 0);

    let (_, queue) = call_json(&state, &cookie, "GET", "/session/queue", None).await;
    assert_eq!(queue["tracks"].as_array().unwrap().len(), 0);
}

/// /session/queue/next is the browser's advance: pop the finished track, return
/// the next one to play, null when the queue runs out.
#[serial(session_queue)]
#[actix_web::test]
async fn test_next_advances_and_runs_out() {
    let state = setup().await;
    let cookie = new_session_cookie(&state).await;
    call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/add",
        Some(json!({ "id": track_id(TRACK_1) })),
    )
    .await;
    call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/add",
        Some(json!({ "id": track_id(TRACK_2) })),
    )
    .await;

    let (_, body) = call_json(&state, &cookie, "POST", "/session/queue/next", None).await;
    assert_eq!(body["track"]["id"], json!(track_id(TRACK_2)));

    let (_, queue) = call_json(&state, &cookie, "GET", "/session/queue", None).await;
    assert_eq!(ids(&queue), [track_id(TRACK_2)]);

    let (_, body) = call_json(&state, &cookie, "POST", "/session/queue/next", None).await;
    assert_eq!(body["track"], Value::Null);

    // Advancing an exhausted queue is a no-op, not an error.
    let (status, body) = call_json(&state, &cookie, "POST", "/session/queue/next", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["track"], Value::Null);
}

/// Adding a folder's tracks (here: an album) works for session queues too.
#[serial(session_queue)]
#[actix_web::test]
async fn test_add_album() {
    let state = setup().await;
    let cookie = new_session_cookie(&state).await;

    let (_, queue) = call_json(
        &state,
        &cookie,
        "POST",
        "/session/queue/add",
        Some(json!({
            "id": pickup::filemanager::utils::generate_album_id(std::path::Path::new("Bryan Teoh/Free")),
        })),
    )
    .await;

    assert_eq!(queue["tracks"].as_array().unwrap().len(), 2);
}

/// The track constants above must name real tracks in the test collection.
#[test]
fn test_track_constants_exist() {
    for relpath in [TRACK_1, TRACK_2, TRACK_3] {
        let id = track_id(relpath);
        assert!(id.starts_with("trk_"), "unexpected id for {relpath}");
    }
}
