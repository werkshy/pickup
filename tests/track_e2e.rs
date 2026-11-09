use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use actix_web::{
    http::{header::CONTENT_RANGE, header::CONTENT_TYPE, header::RANGE, StatusCode},
    test::{self, call_service},
    web::Data,
};
use pickup::{
    app_state::AppState,
    build_app,
    filemanager::{
        collection::Collection,
        model::{CollectionLocation, Kind, Track},
        options::CollectionOptions,
    },
    queue::PlaybackQueue,
    session::SessionStore,
};
use serde_json::{self, json, Value};
use serial_test::serial;

mod helpers;

const TRACK_FILE: &str = "Bryan Teoh/Free/01 - Finally See The Light.mp3";

fn track_id(relpath: &str) -> String {
    pickup::filemanager::utils::generate_track_id(Path::new(relpath))
}

/**
 * Tests run serially: building the collection (and its cache) is not safe
 * to race against, and the missing-file test builds its own collection.
 */
async fn setup() -> Data<AppState> {
    helpers::build_test_app_state()
}

async fn get_track(
    state: &Data<AppState>,
    uri: &str,
    range: Option<&str>,
) -> (StatusCode, String, Option<String>, actix_web::web::Bytes) {
    let app = test::init_service(build_app(state.clone())).await;
    let mut req = test::TestRequest::get().uri(uri);
    if let Some(range) = range {
        req = req.insert_header((RANGE, range));
    }
    let response = call_service(&app, req.to_request()).await;
    let status = response.status();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let content_range = response
        .headers()
        .get(CONTENT_RANGE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    // The opaque body type's error is not Debug, so match instead of unwrap.
    let body = match actix_web::body::to_bytes(response.into_body()).await {
        Ok(bytes) => bytes,
        Err(_) => panic!("failed to read the response body"),
    };
    (status, content_type, content_range, body)
}

/// A full request answers with the whole file, typed by its extension.
#[serial(track)]
#[actix_web::test]
async fn test_serves_the_whole_file() {
    let state = setup().await;

    let (status, content_type, _, body) =
        get_track(&state, &format!("/track/{}", track_id(TRACK_FILE)), None).await;

    assert_eq!(status, StatusCode::OK);
    let expected = std::fs::read(PathBuf::from(helpers::TEST_MUSIC_DIR).join(TRACK_FILE)).unwrap();
    assert_eq!(content_type, "audio/mpeg");
    assert_eq!(body.as_ref(), expected.as_slice());
}

/// A Range request gets the requested bytes back as partial content, which
/// is what makes seeking work in the browser.
#[serial(track)]
#[actix_web::test]
async fn test_range_request_gets_partial_content() {
    let state = setup().await;
    let expected = std::fs::read(PathBuf::from(helpers::TEST_MUSIC_DIR).join(TRACK_FILE)).unwrap();

    let (status, _, content_range, body) = get_track(
        &state,
        &format!("/track/{}", track_id(TRACK_FILE)),
        Some("bytes=100-199"),
    )
    .await;

    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        content_range.unwrap(),
        format!("bytes 100-199/{}", expected.len())
    );
    assert_eq!(body.as_ref(), &expected[100..200]);
}

/// A range beyond the end of the file is invalid.
#[serial(track)]
#[actix_web::test]
async fn test_range_past_the_end_of_the_file_is_unsatisfiable() {
    let state = setup().await;
    let expected = std::fs::read(PathBuf::from(helpers::TEST_MUSIC_DIR).join(TRACK_FILE)).unwrap();

    let (status, _, _, _) = get_track(
        &state,
        &format!("/track/{}", track_id(TRACK_FILE)),
        Some(&format!("bytes={}-", expected.len() + 1000)),
    )
    .await;

    assert_eq!(status, StatusCode::RANGE_NOT_SATISFIABLE);
}

#[serial(track)]
#[actix_web::test]
async fn test_unknown_track_is_not_found() {
    let state = setup().await;

    let (status, _, _, body) = get_track(&state, "/track/trk_nope", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!({
            "code": "not_found",
            "message": "No matching music found in the collection"
        })
    );
}

/// A track whose file disappeared after the collection scan answers with an
/// error, not a panic.
#[serial(track)]
#[actix_web::test]
async fn test_missing_file_is_an_error() {
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
        path: PathBuf::from("definitely/missing/file.wav"),
        category: missing_location.category.clone(),
        artist: missing_location.artist.clone(),
        album: missing_location.album.clone(),
        disc: None,
    };
    let mut collection = Collection::new(String::from(helpers::TEST_MUSIC_DIR));
    collection.add_folder(
        Kind::Album,
        &missing_location,
        Path::new("definitely/missing"),
    );
    collection.add_track(missing_track, &missing_location);

    let queue = Arc::new(RwLock::new(PlaybackQueue::new()));
    let state = Data::new(AppState::new(
        // The player is not involved in serving track files.
        pickup::spawn_player(queue.clone(), PathBuf::from(helpers::TEST_MUSIC_DIR)),
        collection,
        queue,
        SessionStore::new(),
        CollectionOptions {
            dir: String::from(helpers::TEST_MUSIC_DIR),
            ignores: None,
        },
    ));

    let (status, _, _, body) = get_track(&state, "/track/missing", None).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["code"], json!("internal"));
}
