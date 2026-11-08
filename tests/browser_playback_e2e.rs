/**
 * E2E tests for browser playback mode (the default): the /web actions drive
 * the session's queue instead of the jukebox, pages render the session
 * queue in the drawer, and the player bar is the browser-driven flavor.
 * The mode cookie switches between the two.
 */
use std::path::Path;

use actix_web::{
    cookie::Cookie,
    http::StatusCode,
    test::{self, call_service},
    web::Data,
};
use pickup::{app_state::AppState, build_app, session::SESSION_COOKIE, webui::MODE_COOKIE};
use serde_json::{json, Value};
use serial_test::serial;

mod helpers;

const TRACK_1: &str = "Bryan Teoh/Free/01 - Finally See The Light.mp3";
const TRACK_2: &str = "Bryan Teoh/Free/02 - Neptunian Princess.mp3";
const TRACK_1_TITLE: &str = "01 - Finally See The Light";
const TRACK_2_TITLE: &str = "02 - Neptunian Princess";
const ALBUM: &str = "Bryan Teoh/Free";

fn track_id(relpath: &str) -> String {
    pickup::filemanager::utils::generate_track_id(Path::new(relpath))
}

/**
 * One browser session per test: the middleware issues the session cookie on
 * the first request, and the test carries it from there so every request
 * addresses the same session queue. Tests run serially because building the
 * collection (and its cache) is not safe to race against.
 */
async fn setup() -> Data<AppState> {
    helpers::build_test_app_state()
}

async fn new_session(state: &Data<AppState>) -> Cookie<'static> {
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

fn mode_cookie(mode: &str) -> Cookie<'static> {
    Cookie::new(MODE_COOKIE, mode.to_string())
}

struct Posted {
    status: StatusCode,
    body: String,
    set_cookies: Vec<Cookie<'static>>,
}

impl Posted {
    fn assert_ok(&self) {
        assert_eq!(self.status, StatusCode::OK);
    }

    /// The mode cookie the response set, if any.
    fn mode_cookie(&self) -> Option<&str> {
        self.set_cookies
            .iter()
            .find(|cookie| cookie.name() == MODE_COOKIE)
            .map(|cookie| cookie.value())
    }
}

async fn get_html(
    state: &Data<AppState>,
    session: &Cookie<'static>,
    uri: &str,
) -> (StatusCode, String) {
    let app = test::init_service(build_app(state.clone())).await;
    let req = test::TestRequest::get()
        .uri(uri)
        .cookie(session.clone())
        .to_request();
    let response = call_service(&app, req).await;
    let status = response.status();
    let body = test::read_body(response).await;
    (status, String::from_utf8_lossy(&body).to_string())
}

async fn post_html(state: &Data<AppState>, session: &Cookie<'static>, uri: &str) -> Posted {
    let app = test::init_service(build_app(state.clone())).await;
    let req = test::TestRequest::post()
        .uri(uri)
        .cookie(session.clone())
        .to_request();
    let response = call_service(&app, req).await;
    let status = response.status();
    let set_cookies: Vec<Cookie<'static>> = response
        .response()
        .cookies()
        .map(|cookie| cookie.into_owned())
        .collect();
    let body = test::read_body(response).await;
    Posted {
        status,
        body: String::from_utf8_lossy(&body).to_string(),
        set_cookies,
    }
}

async fn get_json(state: &Data<AppState>, session: &Cookie<'static>, uri: &str) -> Value {
    let app = test::init_service(build_app(state.clone())).await;
    let req = test::TestRequest::get()
        .uri(uri)
        .cookie(session.clone())
        .to_request();
    test::call_and_read_body_json(&app, req).await
}

/// With no mode cookie, the UI is in browser mode: the bar carries the
/// browser player's data attributes and no server-side progress poll.
#[serial(browser_playback)]
#[actix_web::test]
async fn test_default_mode_is_browser() {
    let state = setup().await;
    let session = new_session(&state).await;

    let (_, body) = get_html(&state, &session, "/").await;

    assert!(
        body.contains(r#"data-mode="browser""#),
        "the bar is in browser mode: {body}"
    );
    assert!(
        !body.contains(r#"hx-get="/web/fragments/progress""#),
        "the browser bar does not poll the server for progress: {body}"
    );
    assert!(
        body.contains(r#"id="playback-mode""#),
        "the header has the mode switch: {body}"
    );
    assert!(
        body.contains(r#"hx-post="/web/mode/browser""#),
        "the browser side of the switch is there: {body}"
    );
    assert!(
        body.contains(r#"hx-post="/web/mode/jukebox""#),
        "the jukebox side of the switch is there: {body}"
    );
    // The browser side is the active one by default (it is the first of the
    // two buttons in the rendered switch).
    let start = body
        .find(r#"class="mode-option"#)
        .expect("the mode switch renders");
    assert!(
        body[start..].starts_with(r#"class="mode-option active""#),
        "browser mode is the active side by default: {body}"
    );
    assert!(
        body.contains("Nothing playing"),
        "an empty session queue shows nothing playing: {body}"
    );
}

/// The play action replaces the session queue; the jukebox queue and player
/// stay untouched. The swapped bar tells the browser player to start the
/// track.
#[serial(browser_playback)]
#[actix_web::test]
async fn test_play_action_targets_the_session_queue() {
    let state = setup().await;
    let session = new_session(&state).await;

    let posted = post_html(
        &state,
        &session,
        &format!("/web/play/{}", track_id(TRACK_1)),
    )
    .await;
    posted.assert_ok();
    let body = posted.body;

    assert!(
        body.contains(r#"id="queue-list" hx-swap-oob="true""#),
        "the response swaps the drawer out-of-band: {body}"
    );
    assert!(
        body.contains(TRACK_1_TITLE),
        "the queued track is in the drawer: {body}"
    );
    assert!(
        body.contains(&format!(r#"data-track-id="{}""#, track_id(TRACK_1))),
        "the bar names the front track: {body}"
    );
    assert!(
        body.contains(r#"data-autoplay="true""#),
        "playing starts the browser player: {body}"
    );

    // The jukebox is untouched.
    let queue = get_json(&state, &session, "/queue").await;
    assert_eq!(queue["tracks"].as_array().unwrap().len(), 0);
    let status = get_json(&state, &session, "/status").await;
    assert_eq!(status["queued"], 0);

    // The session queue has the track.
    let session_queue = get_json(&state, &session, "/session/queue").await;
    assert_eq!(session_queue["tracks"].as_array().unwrap().len(), 1);

    // Clean the session up (it would linger until the TTL otherwise).
    let _ = post_html(&state, &session, "/web/queue/clear").await;
}

/// The add and next actions queue behind the session queue's front.
#[serial(browser_playback)]
#[actix_web::test]
async fn test_add_and_next_actions_target_the_session_queue() {
    let state = setup().await;
    let session = new_session(&state).await;

    post_html(
        &state,
        &session,
        &format!("/web/play/{}", track_id(TRACK_1)),
    )
    .await
    .assert_ok();

    let posted = post_html(
        &state,
        &session,
        &format!("/web/queue/next/{}", track_id(TRACK_2)),
    )
    .await;
    posted.assert_ok();
    let body = posted.body;
    assert!(
        body.contains(TRACK_2_TITLE),
        "the queued track is in the drawer: {body}"
    );

    let session_queue = get_json(&state, &session, "/session/queue").await;
    let tracks = session_queue["tracks"].as_array().unwrap();
    assert_eq!(tracks.len(), 2);
    assert_eq!(tracks[0]["id"], json!(track_id(TRACK_1)));
    assert_eq!(tracks[1]["id"], json!(track_id(TRACK_2)));

    // The add action appends to the session queue too.
    post_html(
        &state,
        &session,
        &format!("/web/queue/add/{}", track_id(TRACK_2)),
    )
    .await
    .assert_ok();
    let session_queue = get_json(&state, &session, "/session/queue").await;
    assert_eq!(session_queue["tracks"].as_array().unwrap().len(), 3);

    let _ = post_html(&state, &session, "/web/queue/clear").await;
}

/// Full page renders show the session queue in the drawer. The server never
/// marks anything as playing in browser mode (it cannot know whether the
/// browser is sounding it): the player script owns that highlight.
#[serial(browser_playback)]
#[actix_web::test]
async fn test_pages_render_the_session_queue() {
    let state = setup().await;
    let session = new_session(&state).await;

    post_html(
        &state,
        &session,
        &format!("/web/play/{}", track_id(TRACK_2)),
    )
    .await
    .assert_ok();

    let album = pickup::filemanager::utils::generate_album_id(Path::new(ALBUM));
    let (_, body) = get_html(&state, &session, &format!("/browse/{album}")).await;

    assert!(
        body.contains(TRACK_2_TITLE),
        "the session queue is in the drawer: {body}"
    );
    assert!(
        !body.contains(r#"class="playing""#),
        "the server must not fake a playing highlight: {body}"
    );

    let _ = post_html(&state, &session, "/web/queue/clear").await;
}

/// The queue fragment (which the player script swaps in after an advance)
/// renders the session queue in browser mode.
#[serial(browser_playback)]
#[actix_web::test]
async fn test_queue_fragment_renders_the_session_queue() {
    let state = setup().await;
    let session = new_session(&state).await;

    post_html(
        &state,
        &session,
        &format!("/web/play/{}", track_id(TRACK_1)),
    )
    .await
    .assert_ok();

    let (_, body) = get_html(&state, &session, "/fragments/queue").await;
    assert!(
        body.contains(TRACK_1_TITLE),
        "the fragment shows the session queue: {body}"
    );
    assert!(
        !body.contains("hx-swap-oob"),
        "a direct swap must not carry the OOB marker: {body}"
    );

    let _ = post_html(&state, &session, "/web/queue/clear").await;
}

/// Switching modes sets the mode cookie and re-renders the chrome for the
/// mode switched to. The mode endpoints are explicit (the header switch
/// posts the side it should activate), not a blind toggle.
#[serial(browser_playback)]
#[actix_web::test]
async fn test_mode_switch_sets_the_cookie_and_rerenders() {
    let state = setup().await;
    let session = new_session(&state).await;

    // Browser mode is the default; switching to it when already there does
    // not re-set the cookie.
    let posted = post_html(&state, &session, "/web/mode/browser").await;
    posted.assert_ok();
    assert_eq!(posted.mode_cookie(), None, "no switch, no new cookie");

    let posted = post_html(&state, &session, "/web/mode/jukebox").await;
    posted.assert_ok();
    assert_eq!(posted.mode_cookie(), Some("jukebox"));
    let body = posted.body;
    assert!(
        body.contains(r#"data-mode="jukebox""#),
        "the chrome re-renders for jukebox mode: {body}"
    );
    assert!(
        body.contains(r#"hx-get="/web/fragments/progress""#),
        "the jukebox bar carries the progress poll: {body}"
    );
    // The mode switch renders with the jukebox side active.
    assert!(
        body.contains(r#"hx-swap-oob="true""#) && body.contains(r#"id="playback-mode""#),
        "the response swaps the mode switch out-of-band: {body}"
    );

    // With the mode cookie set, pages render in jukebox mode.
    let jukebox = mode_cookie("jukebox");
    let (_, body) = get_html(&state, &jukebox, "/").await;
    assert!(
        body.contains(r#"data-mode="jukebox""#),
        "the cookie persists the mode: {body}"
    );

    // Switching back to browser mode.
    let posted = post_html(&state, &jukebox, "/web/mode/browser").await;
    assert_eq!(posted.mode_cookie(), Some("browser"));
}

/// In jukebox mode the session queue stays untouched, and vice versa: the
/// two modes do not sync (by design).
#[serial(browser_playback)]
#[actix_web::test]
async fn test_modes_do_not_sync() {
    let state = setup().await;
    let session = new_session(&state).await;

    // A browser-mode play.
    post_html(
        &state,
        &session,
        &format!("/web/play/{}", track_id(TRACK_1)),
    )
    .await
    .assert_ok();

    // A jukebox-mode clear (with the mode cookie pinned).
    let app = test::init_service(build_app(state.clone())).await;
    let req = test::TestRequest::post()
        .uri("/web/queue/clear")
        .cookie(mode_cookie("jukebox"))
        .to_request();
    let response = call_service(&app, req).await;
    assert_eq!(response.status(), StatusCode::OK);

    // The session queue survives the jukebox clear.
    let session_queue = get_json(&state, &session, "/session/queue").await;
    assert_eq!(session_queue["tracks"].as_array().unwrap().len(), 1);

    let _ = post_html(&state, &session, "/web/queue/clear").await;
}

/// Switching to browser mode stops the server player: its playback would
/// otherwise keep sounding with no controls in the UI.
#[serial(browser_playback)]
#[actix_web::test]
async fn test_switching_to_browser_mode_stops_the_server_player() {
    let state = setup().await;

    // Play a track on the jukebox (pinned jukebox mode).
    let app = test::init_service(build_app(state.clone())).await;
    let req = test::TestRequest::post()
        .uri(&format!("/web/play/{}", track_id(TRACK_1)))
        .cookie(mode_cookie("jukebox"))
        .to_request();
    let response = call_service(&app, req).await;
    assert_eq!(response.status(), StatusCode::OK);
    let jukebox = mode_cookie("jukebox");
    let status = get_json(&state, &jukebox, "/status").await;
    assert!(
        status["position_secs"].is_number(),
        "the jukebox is playing: {status}"
    );

    // Toggle to browser mode (the request must carry jukebox mode to switch
    // out of it); the server player must have stopped.
    let session = new_session(&state).await;
    let app = test::init_service(build_app(state.clone())).await;
    let req = test::TestRequest::post()
        .uri("/web/mode/browser")
        .cookie(mode_cookie("jukebox"))
        .to_request();
    let response = call_service(&app, req).await;
    assert_eq!(response.status(), StatusCode::OK);

    let status = get_json(&state, &jukebox, "/status").await;
    assert!(
        status["position_secs"].is_null(),
        "the jukebox must be stopped after the switch: {status}"
    );

    let _ = post_html(&state, &session, "/web/queue/clear").await;
}
