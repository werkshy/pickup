/**
 * E2E tests for the web UI: pages render, actions share the JSON API's
 * queue behavior, and action responses carry the refreshed drawer as an
 * out-of-band swap.
 */
use std::path::Path;

use actix_web::{
    http::{header::CONTENT_TYPE, StatusCode},
    test,
};
use once_cell::sync::Lazy;
use serial_test::serial;

mod helpers;

use helpers::build_test_app_state;
use pickup::{
    app_state::AppState,
    build_app,
    filemanager::utils::{
        generate_album_id, generate_artist_id, generate_category_id, generate_track_id,
    },
};

static APP_STATE: Lazy<actix_web::web::Data<AppState>> = Lazy::new(build_test_app_state);

async fn get_html(uri: &str) -> (StatusCode, String) {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::get().uri(uri).to_request();
    let resp = test::call_service(&app, req).await;
    let content_type = resp
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    let status = resp.status();
    let body = test::read_body(resp).await;
    (
        status,
        format!("{content_type}\n{}", String::from_utf8_lossy(&body)),
    )
}

async fn post_html(uri: &str) -> (StatusCode, String) {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post().uri(uri).to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body = test::read_body(resp).await;
    (status, String::from_utf8_lossy(&body).to_string())
}

async fn get_json(uri: &str) -> serde_json::Value {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::get().uri(uri).to_request();
    test::call_and_read_body_json(&app, req).await
}

async fn clear_playlist() {
    let _ = post_html("/web/queue/clear").await;
}

const TRACK_1: &str = "Bryan Teoh/Free/01 - Finally See The Light.mp3";
const TRACK_2: &str = "Bryan Teoh/Free/02 - Neptunian Princess.mp3";
const TRACK_2_TITLE: &str = "02 - Neptunian Princess";
const TRACK_1_TITLE: &str = "01 - Finally See The Light";
const ALBUM: &str = "Bryan Teoh/Free";
const ARTIST: &str = "Bryan Teoh";

#[actix_web::test]
async fn test_home_page_lists_the_categories() {
    let (status, body) = get_html("/").await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body.starts_with("text/html"),
        "the home page is HTML: {body}"
    );
    assert!(
        body.contains(&generate_category_id(Path::new("Music"))),
        "the Music category tab should be on the home page: {body}"
    );
    assert!(body.contains(">Music</a>"), "unexpected body: {body}");
}

#[actix_web::test]
async fn test_browse_category_page_lists_artists() {
    let category = generate_category_id(Path::new("Music"));
    let (status, body) = get_html(&format!("/browse/{category}")).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(">Artists</h2>"), "unexpected body: {body}");
    assert!(
        body.contains(&generate_artist_id(Path::new(ARTIST))),
        "the artist link should be on the category page: {body}"
    );
}

#[actix_web::test]
async fn test_browse_artist_page_lists_albums() {
    let artist = generate_artist_id(Path::new(ARTIST));
    let (status, body) = get_html(&format!("/browse/{artist}")).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(">Albums</h2>"), "unexpected body: {body}");
    assert!(
        body.contains(&generate_album_id(Path::new(ALBUM))),
        "the album link should be on the artist page: {body}"
    );
}

#[actix_web::test]
async fn test_browse_album_page_lists_tracks() {
    let album = generate_album_id(Path::new(ALBUM));
    let (status, body) = get_html(&format!("/browse/{album}")).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(">Tracks</h2>"), "unexpected body: {body}");
    assert!(
        body.contains(TRACK_2_TITLE),
        "the track should be on the album page: {body}"
    );
    assert!(
        body.contains(&generate_track_id(Path::new(TRACK_2))),
        "the track's actions should address its ID: {body}"
    );
}

#[actix_web::test]
async fn test_browse_unknown_folder_is_404() {
    let (status, body) = get_html("/browse/cat_nope").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        body.starts_with("text/html"),
        "a browser gets the HTML error page: {body}"
    );
    assert!(
        body.contains("Page not found"),
        "unexpected error page: {body}"
    );
    assert!(
        body.contains("/browse/cat_nope"),
        "the error page names the requested path: {body}"
    );
}

/// The add action must return the refreshed drawer as an out-of-band swap,
/// and the queue it reports must match the JSON API's view.
#[serial(queue)]
#[actix_web::test]
async fn test_add_action_returns_an_oob_queue_fragment() {
    clear_playlist().await;

    let album = generate_album_id(Path::new(ALBUM));
    let (status, body) = post_html(&format!("/web/queue/add/{album}")).await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains(r#"id="queue-list" hx-swap-oob="true""#),
        "the response must be an OOB swap of the queue list: {body}"
    );
    assert!(
        body.contains(TRACK_2_TITLE),
        "the added track should be in the fragment: {body}"
    );

    let queue = get_json("/queue").await;
    assert_eq!(queue["tracks"].as_array().unwrap().len(), 2);

    clear_playlist().await;
}

/// Play next must queue behind the playing track.
#[serial(queue)]
#[actix_web::test]
async fn test_next_action_queues_behind_the_playing_track() {
    clear_playlist().await;

    let play = format!("/web/play/{}", generate_track_id(Path::new(TRACK_1)));
    let (status, _) = post_html(&play).await;
    assert_eq!(status, StatusCode::OK);

    let next = format!("/web/queue/next/{}", generate_track_id(Path::new(TRACK_2)));
    let (status, body) = post_html(&next).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains(TRACK_2_TITLE),
        "the queued track should be in the fragment: {body}"
    );

    let queue = get_json("/queue").await;
    let tracks = queue["tracks"].as_array().unwrap();
    assert_eq!(tracks.len(), 2);
    assert_eq!(tracks[0]["id"], generate_track_id(Path::new(TRACK_1)));
    assert_eq!(tracks[1]["id"], generate_track_id(Path::new(TRACK_2)));

    clear_playlist().await;
}

/// The play action replaces the queue and starts playback.
#[serial(queue)]
#[actix_web::test]
async fn test_play_action_replaces_and_starts() {
    clear_playlist().await;

    let album = generate_album_id(Path::new(ALBUM));
    let _ = post_html(&format!("/web/queue/add/{album}")).await;

    let play = format!("/web/play/{}", generate_track_id(Path::new(TRACK_2)));
    let (status, _) = post_html(&play).await;
    assert_eq!(status, StatusCode::OK);

    let status_json = get_json("/status").await;
    assert_eq!(status_json["queued"], 1);

    let queue = get_json("/queue").await;
    let tracks = queue["tracks"].as_array().unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0]["id"], generate_track_id(Path::new(TRACK_2)));

    clear_playlist().await;
}

/// The remove action goes through the same operation as the JSON API:
/// removing a queued track leaves playback alone.
#[serial(queue)]
#[actix_web::test]
async fn test_remove_action_removes_a_queued_track() {
    clear_playlist().await;

    let album = generate_album_id(Path::new(ALBUM));
    let _ = post_html(&format!("/web/queue/add/{album}")).await;

    let (status, body) = post_html("/web/queue/remove/1").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("Finally See The Light"),
        "the remaining track should still be in the fragment: {body}"
    );

    let queue = get_json("/queue").await;
    assert_eq!(queue["tracks"].as_array().unwrap().len(), 1);

    clear_playlist().await;
}

#[serial(queue)]
#[actix_web::test]
async fn test_clear_action_empties_the_queue() {
    clear_playlist().await;

    let album = generate_album_id(Path::new(ALBUM));
    let _ = post_html(&format!("/web/queue/add/{album}")).await;

    let (status, body) = post_html("/web/queue/clear").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("The queue is empty"),
        "the empty queue should be in the fragment: {body}"
    );

    let queue = get_json("/queue").await;
    assert_eq!(queue["tracks"].as_array().unwrap().len(), 0);
}

/// The fragment endpoint serves the drawer list without the OOB marker, so
/// it can be swapped into the drawer directly (e.g. by a poll later).
#[serial(queue)]
#[actix_web::test]
async fn test_queue_fragment_is_not_oob() {
    clear_playlist().await;

    let track = generate_track_id(Path::new(TRACK_1));
    let _ = post_html(&format!("/web/queue/add/{track}")).await;

    let (status, body) = get_html("/fragments/queue").await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body.starts_with("text/html"),
        "the fragment is HTML: {body}"
    );
    assert!(
        !body.contains("hx-swap-oob"),
        "a direct swap must not carry the OOB marker: {body}"
    );
    assert!(
        body.contains("Finally See The Light"),
        "the queued track should be in the fragment: {body}"
    );

    clear_playlist().await;
}

/// The JSON body of an action error stays in the standard error shape; htmx
/// will ignore non-2xx responses for now.
#[serial(queue)]
#[actix_web::test]
async fn test_action_with_unknown_id_is_404() {
    clear_playlist().await;

    let (status, body) = post_html("/web/queue/add/cat_nope").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains("No matching music found"), "{body}");
}

#[actix_web::test]
async fn test_static_assets_are_served() {
    let (status, body) = get_html("/static/htmx.min.js").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("htmx"), "htmx should be served from /static");

    let (status, _) = get_html("/static/app.css").await;
    assert_eq!(status, StatusCode::OK);

    // The favicon is served in both the modern (svg) and legacy (ico) forms.
    let (status, body) = get_html("/static/favicon.svg").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("<svg"), "favicon.svg is an svg: {body}");

    let (status, _) = get_html("/static/favicon.ico").await;
    assert_eq!(status, StatusCode::OK);
}

/// The queue drawer shows the JSON-visible queue state on full page loads,
/// including which track is playing.
#[serial(queue)]
#[actix_web::test]
async fn test_album_page_shows_the_queue_in_the_drawer() {
    clear_playlist().await;

    let track = generate_track_id(Path::new(TRACK_2));
    let _ = post_html(&format!("/web/play/{track}")).await;

    let album = generate_album_id(Path::new(ALBUM));
    let (_, body) = get_html(&format!("/browse/{album}")).await;

    assert!(
        body.contains(r#"id="queue-list""#),
        "the drawer renders with the page: {body}"
    );
    assert!(
        body.contains(TRACK_2_TITLE),
        "the playing track should be in the drawer: {body}"
    );
    assert!(
        body.contains(r#"class="playing""#),
        "the playing track is highlighted: {body}"
    );

    clear_playlist().await;
}

/// The volume action posts a form-encoded value (htmx sends the slider's
/// name/value as an urlencoded body).
async fn post_form(uri: &str, body: &str) -> (StatusCode, String) {
    let app = test::init_service(build_app((*APP_STATE).clone())).await;
    let req = test::TestRequest::post()
        .uri(uri)
        .insert_header((CONTENT_TYPE, "application/x-www-form-urlencoded"))
        .set_payload(body.to_string())
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body = test::read_body(resp).await;
    (status, String::from_utf8_lossy(&body).to_string())
}

/// Every page renders the player bar in the shell, with the poll attributes
/// and the transport in its idle state.
#[actix_web::test]
async fn test_player_bar_renders_on_full_pages() {
    let (_, body) = get_html("/").await;

    assert!(
        body.contains(r#"id="player-bar""#),
        "the bar is in the shell: {body}"
    );
    assert!(
        body.contains(r#"hx-get="/web/fragments/progress""#),
        "the bar polls the progress fragment: {body}"
    );
    assert!(
        body.contains(r#"id="progress""#),
        "the poll target exists on the page (htmx targetError otherwise): {body}"
    );
    assert!(
        body.contains(r#"id="volume""#),
        "the bar has the volume slider: {body}"
    );
    assert!(
        body.contains("Nothing playing"),
        "the idle bar shows nothing playing: {body}"
    );
    assert!(
        body.contains(r#"id="transport-play""#),
        "the idle bar shows the play (resume) button: {body}"
    );
}

/// The transport buttons swap the bar out-of-band: pause flips the toggle,
/// resume flips it back, stop idles the bar.
#[serial(queue)]
#[actix_web::test]
async fn test_transport_actions_swap_the_bar_out_of_band() {
    clear_playlist().await;

    let track = generate_track_id(Path::new(TRACK_1));
    let (_, body) = post_html(&format!("/web/play/{track}")).await;
    assert!(
        body.contains(r#"id="player-bar" hx-swap-oob"#),
        "playing swaps the bar out-of-band: {body}"
    );
    assert!(
        body.contains(r#"id="transport-pause""#),
        "the playing bar shows the pause button: {body}"
    );
    assert!(
        body.contains(TRACK_1_TITLE),
        "the bar shows the playing track: {body}"
    );

    let (_, body) = post_html("/web/pause").await;
    assert!(
        body.contains(r#"id="transport-play""#),
        "the paused bar shows the play button: {body}"
    );

    let (_, body) = post_html("/web/resume").await;
    assert!(
        body.contains(r#"id="transport-pause""#),
        "the resumed bar shows the pause button: {body}"
    );

    let (_, body) = post_html("/web/stop").await;
    assert!(
        body.contains("Nothing playing"),
        "the stopped bar shows nothing playing: {body}"
    );
    assert!(
        body.contains(r#"id="transport-play""#),
        "the stopped bar shows the play button: {body}"
    );

    clear_playlist().await;
}

/// Skip advances the queue: the swapped bar shows the new track and the
/// swapped drawer marks it as playing.
#[serial(queue)]
#[actix_web::test]
async fn test_skip_action_swaps_the_bar_and_the_drawer() {
    clear_playlist().await;

    let album = generate_album_id(Path::new(ALBUM));
    let _ = post_html(&format!("/web/queue/add/{album}")).await;

    let (_, body) = post_html("/web/skip").await;
    assert!(
        body.contains(TRACK_1_TITLE),
        "the bar shows the track skip started: {body}"
    );
    assert!(
        body.contains(r#"id="player-bar" hx-swap-oob"#),
        "skip swaps the bar: {body}"
    );
    assert!(
        body.contains(r#"id="queue-list" hx-swap-oob"#),
        "skip swaps the drawer: {body}"
    );

    clear_playlist().await;
}

/// The volume action renders the value that was actually set into the
/// swapped bar's slider.
#[serial(queue)]
#[actix_web::test]
async fn test_volume_action_sets_the_slider() {
    let (_, body) = post_form("/web/volume", "volume=42").await;

    assert!(body.contains(r#"id="player-bar" hx-swap-oob"#), "{body}");
    assert!(
        body.contains(r#"name="volume" value="42""#),
        "the swapped bar carries the set volume: {body}"
    );
}

/// The volume slider clamps out-of-range values server-side.
#[serial(queue)]
#[actix_web::test]
async fn test_volume_action_clamps_out_of_range_values() {
    let (_, body) = post_form("/web/volume", "volume=4000").await;

    assert!(
        body.contains(r#"name="volume" value="100""#),
        "4000 clamps to 100: {body}"
    );
}

/// The progress fragment is the poll target: it renders the elapsed readout
/// while playing, and is empty while idle.
#[serial(queue)]
#[actix_web::test]
async fn test_progress_fragment_shows_the_elapsed_time() {
    clear_playlist().await;

    let (_, idle) = get_html("/web/fragments/progress").await;
    assert!(
        idle.contains(r#"<span id="progress"></span>"#),
        "the idle progress is empty: {idle}"
    );

    let track = generate_track_id(Path::new(TRACK_1));
    let _ = post_html(&format!("/web/play/{track}")).await;

    let (_, playing) = get_html("/web/fragments/progress").await;
    assert!(
        playing.contains(r#"id="progress""#),
        "the playing progress renders: {playing}"
    );
    assert!(
        !playing.contains(r#"hx-swap-oob"#),
        "the poll swap is direct, not out-of-band: {playing}"
    );
    // The readout is elapsed / total, e.g. "0:00 / 0:15".
    assert!(
        playing.contains(" / "),
        "the readout is elapsed / total: {playing}"
    );

    clear_playlist().await;
}
