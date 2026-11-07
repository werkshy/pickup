use std::path::Path;

use actix_web::{http::StatusCode, test, web::Data};

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

// Share one app state across the tests, because Sled won't let us open
// multiple instances of the DB file from different threads.
static APP_STATE: Lazy<Data<AppState>> = Lazy::new(build_test_app_state);

#[actix_web::test]
async fn test_list_categories() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::get().uri("/categories").to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(
        resp,
        json!({ "categories": [
            {
                "id": generate_category_id(Path::new("Music")),
                "name": "Music",
                "kind": "category",
            },
            {
                "id": generate_category_id(Path::new("_Free")),
                "name": "Free",
                "kind": "category",
            },
        ] })
    );
}

/**
 * A category folder lists the artists under it, so the collection can be
 * walked one `/folders/{id}` at a time.
 */
#[actix_web::test]
async fn test_get_folder_by_id_lists_child_folders() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let id = generate_category_id(Path::new("Music"));
    let req = test::TestRequest::get()
        .uri(format!("/folders/{id}").as_str())
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(
        resp,
        json!({
            "id": id,
            "name": "Music",
            "kind": "category",
            "folders": [
                {
                    "id": generate_artist_id(Path::new("Alex-Productions")),
                    "name": "Alex-Productions",
                    "kind": "artist",
                },
                {
                    "id": generate_artist_id(Path::new("Bryan Teoh")),
                    "name": "Bryan Teoh",
                    "kind": "artist",
                },
                {
                    "id": generate_artist_id(Path::new("Kevin MacLeod")),
                    "name": "Kevin MacLeod",
                    "kind": "artist",
                },
            ],
            "tracks": [],
        })
    );
}

#[actix_web::test]
async fn test_get_folder_by_id_lists_tracks() {
    // An album's immediate contents are its tracks.
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let id = generate_album_id(Path::new("Bryan Teoh/Free"));
    let req = test::TestRequest::get()
        .uri(format!("/folders/{id}").as_str())
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp["id"], json!(id));
    assert_eq!(resp["name"], "Free");
    assert_eq!(resp["kind"], "album");
    assert_eq!(resp["folders"].as_array().unwrap().len(), 0);

    // The cache order isn't asserted, so compare the ID sets.
    let mut track_ids: Vec<String> = resp["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|track| track["id"].as_str().unwrap().to_string())
        .collect();
    track_ids.sort();
    let mut expected = vec![
        generate_track_id(Path::new("Bryan Teoh/Free/01 - Finally See The Light.mp3")),
        generate_track_id(Path::new("Bryan Teoh/Free/02 - Neptunian Princess.mp3")),
    ];
    expected.sort();
    assert_eq!(track_ids, expected);
}

#[actix_web::test]
async fn test_get_folder_by_id_lists_discs() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let id = generate_album_id(Path::new("Kevin MacLeod/Misc"));
    let req = test::TestRequest::get()
        .uri(format!("/folders/{id}").as_str())
        .to_request();

    let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

    assert_eq!(
        resp["folders"],
        json!([
            {
                "id": generate_disc_id(Path::new("Kevin MacLeod/Misc/CD 1")),
                "name": "CD 1",
                "kind": "disc",
            },
            {
                "id": generate_disc_id(Path::new("Kevin MacLeod/Misc/CD 2")),
                "name": "CD 2",
                "kind": "disc",
            },
        ])
    );
    assert_eq!(resp["tracks"].as_array().unwrap().len(), 0);
}

#[actix_web::test]
async fn test_get_folder_by_unknown_id_is_404() {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::get()
        .uri("/folders/cat_nope")
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body,
        json!({ "code": "not_found", "message": "No folder found with ID 'cat_nope'" })
    );
}
