use actix_web::{
    cookie::Cookie,
    test::{self, call_service},
    web::Data,
};
use pickup::{app_state::AppState, build_app, session::SESSION_COOKIE};
use serde_json::json;
use serial_test::serial;

mod helpers;

/**
 * Each test gets a fresh app state, so the session store's contents are
 * deterministic. Tests run serially because building the collection (and
 * its cache) is not safe to race against.
 */
async fn setup() -> Data<AppState> {
    helpers::build_test_app_state()
}

async fn get_hello(
    state: &Data<AppState>,
    cookie: Option<Cookie<'static>>,
) -> (actix_web::http::StatusCode, Option<Cookie<'static>>) {
    let app = test::init_service(build_app(state.clone())).await;
    let mut req = test::TestRequest::get().uri("/hello");
    if let Some(cookie) = cookie {
        req = req.cookie(cookie);
    }
    let response = call_service(&app, req.to_request()).await;
    let status = response.status();
    let session_cookie = session_cookie_of(&response);
    (status, session_cookie)
}

fn session_cookie_of<B: actix_web::body::MessageBody>(
    response: &actix_web::dev::ServiceResponse<B>,
) -> Option<Cookie<'static>> {
    response
        .response()
        .cookies()
        .find(|cookie| cookie.name() == SESSION_COOKIE)
        .map(|cookie| cookie.into_owned())
}

/// A first request without a session cookie must get one: the cookie is set
/// with the right attributes, and the session exists in the store.
#[serial(sessions)]
#[actix_web::test]
async fn test_first_request_sets_a_session_cookie() {
    let state = setup().await;

    let response = get_hello(&state, None).await;
    let (status, maybe_cookie) = response;
    assert_eq!(status, actix_web::http::StatusCode::OK);

    let cookie = maybe_cookie.expect("response sets a session cookie");
    assert_eq!(cookie.name(), SESSION_COOKIE);
    assert_eq!(cookie.value().len(), 32);
    assert!(
        cookie.value().chars().all(|c| c.is_ascii_hexdigit()),
        "unexpected cookie value: {}",
        cookie.value()
    );
    assert_eq!(cookie.path().unwrap(), "/");
    assert!(cookie.http_only().unwrap());
    assert_eq!(
        cookie.same_site().unwrap(),
        actix_web::cookie::SameSite::Lax
    );
    // 30 days, give or take the rounding to whole days in the cookie header.
    assert_eq!(
        cookie.max_age().unwrap(),
        actix_web::cookie::time::Duration::days(30)
    );

    assert_eq!(state.sessions.len(), 1);
}

/// A request that carries the session cookie keeps its session: no new
/// cookie is set and no extra session is created.
#[serial(sessions)]
#[actix_web::test]
async fn test_request_with_a_session_cookie_reuses_the_session() {
    let state = setup().await;

    let first = get_hello(&state, None).await;
    let cookie = first.1.expect("first response sets the session cookie");

    let second = get_hello(&state, Some(cookie)).await;

    assert!(second.1.is_none());
    assert_eq!(state.sessions.len(), 1);
}

/// Two different sessions must not interfere: each cookie addresses its own
/// session in the store.
#[serial(sessions)]
#[actix_web::test]
async fn test_sessions_are_per_cookie() {
    let state = setup().await;

    let first = get_hello(&state, None).await;
    let second = get_hello(&state, None).await;

    let cookie_a = first.1.unwrap();
    let cookie_b = second.1.unwrap();
    assert_ne!(cookie_a.value(), cookie_b.value());

    assert_eq!(state.sessions.len(), 2);
}

/// A malformed session cookie is treated as no cookie: the client gets a
/// fresh session, and the junk value never becomes a session.
#[serial(sessions)]
#[actix_web::test]
async fn test_malformed_cookie_gets_a_new_session() {
    let state = setup().await;
    let junk = Cookie::new(SESSION_COOKIE, "not-a-session-id");

    let response = get_hello(&state, Some(junk)).await;

    let cookie = response.1.expect("response replaces the bad cookie");
    assert_ne!(cookie.value(), "not-a-session-id");
    assert_eq!(state.sessions.len(), 1);
}

/// A well-formed cookie for a session this server process does not remember
/// (e.g. after a restart) is adopted as-is: no new cookie is set, and the
/// session comes back to life.
#[serial(sessions)]
#[actix_web::test]
async fn test_unknown_but_well_formed_cookie_is_adopted() {
    let state = setup().await;
    let id = "0123456789abcdef0123456789abcdef";
    let cookie = Cookie::new(SESSION_COOKIE, id);

    let response = get_hello(&state, Some(cookie)).await;

    assert!(response.1.is_none());
    assert_eq!(state.sessions.len(), 1);
}

/// Non-session cookies do not confuse the middleware.
#[serial(sessions)]
#[actix_web::test]
async fn test_other_cookies_are_ignored() {
    let state = setup().await;
    let unrelated = Cookie::new("other", "value");

    let response = get_hello(&state, Some(unrelated)).await;

    let cookie = response.1.expect("response sets a session cookie");
    assert_eq!(cookie.name(), SESSION_COOKIE);
}

/// The middleware does not break anything downstream: an unknown route
/// still answers with the standard JSON error shape.
#[serial(sessions)]
#[actix_web::test]
async fn test_error_responses_still_work() {
    let state = setup().await;
    let app = test::init_service(build_app(state.clone())).await;
    let req = test::TestRequest::get().uri("/no-such-route").to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), actix_web::http::StatusCode::NOT_FOUND);
    let body: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(body["code"], json!("not_found"));
}
