use std::{
    collections::HashMap,
    future::{ready, Future, Ready},
    pin::Pin,
    sync::{Mutex, RwLock},
    task::{Context, Poll},
    time::{Duration, Instant},
};

use actix_web::{
    body::MessageBody,
    cookie::{time::Duration as CookieDuration, Cookie, SameSite},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web::Data,
    Error, HttpMessage, HttpRequest,
};
use rand::random;

use crate::{app_state::AppState, queue::PlaybackQueue};

pub const SESSION_COOKIE: &str = "pickup_sess";

/**
 * How long a session lives without any activity. This matches the cookie
 * Max-Age, so the browser forgets the session about when the server does.
 */
const SESSION_TTL: Duration = Duration::from_secs(30 * 24 * 60 * 60);
const SESSION_TTL_DAYS: i64 = 30;

/// How often the store looks for and drops expired sessions.
const SWEEP_INTERVAL: Duration = Duration::from_secs(60 * 60);

/**
 * The id of the browser session a request belongs to, inserted into the
 * request extensions by the session middleware. Handlers use it to look up
 * the session's own state (its queue).
 */
pub struct SessionId(pub String);

/// Look up the session id a request belongs to, as inserted by the session
/// middleware.
pub fn session_id(req: &HttpRequest) -> Option<String> {
    req.extensions().get::<SessionId>().map(|id| id.0.clone())
}

struct SessionState {
    queue: PlaybackQueue,
    last_seen: Instant,
}

impl SessionState {
    fn new() -> SessionState {
        SessionState {
            queue: PlaybackQueue::new(),
            last_seen: Instant::now(),
        }
    }
}

/**
 * Every browser gets its own playback queue, keyed by the `pickup_sess`
 * cookie. The store owns all sessions: the middleware keeps each session's
 * `last_seen` fresh on every request, and drops sessions that have been
 * idle past the TTL.
 */
pub struct SessionStore {
    sessions: RwLock<HashMap<String, SessionState>>,
    next_sweep: Mutex<Instant>,
}

impl SessionStore {
    pub fn new() -> SessionStore {
        SessionStore {
            sessions: RwLock::new(HashMap::new()),
            next_sweep: Mutex::new(Instant::now()),
        }
    }

    /**
     * Resolve the session id a request belongs to: the id on the request's
     * cookie when it names a session (even one this server process does not
     * remember, e.g. after a restart), otherwise a freshly created one.
     * Returns the id, and whether the response must set the cookie.
     */
    pub fn resolve(&self, cookie: Option<&str>) -> (String, bool) {
        self.sweep_if_due();
        match cookie.filter(|value| is_valid_id(value)) {
            Some(id) => {
                self.touch(id);
                (id.to_string(), false)
            }
            None => {
                let id = new_id();
                self.touch(&id);
                (id, true)
            }
        }
    }

    /// Mark the session as seen now, creating it if it does not exist.
    fn touch(&self, id: &str) {
        let mut sessions = self.sessions.write().unwrap();
        match sessions.get_mut(id) {
            Some(state) => state.last_seen = Instant::now(),
            None => {
                sessions.insert(id.to_string(), SessionState::new());
            }
        }
    }

    /// Drop expired sessions, but at most once per sweep interval.
    fn sweep_if_due(&self) {
        let mut next_sweep = self.next_sweep.lock().unwrap();
        if next_sweep.elapsed() < SWEEP_INTERVAL {
            return;
        }
        *next_sweep = Instant::now();
        self.sweep_older_than(SESSION_TTL);
    }

    /// Drop sessions that have been idle for at least `max_age`.
    pub fn sweep_older_than(&self, max_age: Duration) {
        self.sessions
            .write()
            .unwrap()
            .retain(|_, state| state.last_seen.elapsed() < max_age);
    }

    /// Number of live sessions. For tests and diagnostics.
    pub fn len(&self) -> usize {
        self.sessions.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Run `f` with read access to the session's queue. Returns None when
    /// the session does not exist.
    pub fn read_queue<R>(&self, id: &str, f: impl FnOnce(&PlaybackQueue) -> R) -> Option<R> {
        self.sessions
            .read()
            .unwrap()
            .get(id)
            .map(|state| f(&state.queue))
    }

    /// Run `f` with mutable access to the session's queue. Returns None
    /// when the session does not exist.
    pub fn update_queue<R>(&self, id: &str, f: impl FnOnce(&mut PlaybackQueue) -> R) -> Option<R> {
        self.sessions
            .write()
            .unwrap()
            .get_mut(id)
            .map(|state| f(&mut state.queue))
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

/**
 * The cookie to set when a client has no session yet. The 30-day Max-Age
 * keeps the session across browser restarts; it is HttpOnly so scripts
 * cannot read it, and SameSite=Lax so it travels with htmx requests.
 */
fn session_cookie(id: &str) -> Cookie<'static> {
    Cookie::build(SESSION_COOKIE, id.to_string())
        .path("/")
        .max_age(CookieDuration::days(SESSION_TTL_DAYS))
        .http_only(true)
        .same_site(SameSite::Lax)
        .finish()
}

/// Session ids are 32 hex characters (128 random bits).
fn is_valid_id(id: &str) -> bool {
    id.len() == 32 && id.chars().all(|c| c.is_ascii_hexdigit())
}

fn new_id() -> String {
    // 128 random bits, 32 hex characters.
    format!("{:032x}", random::<u128>())
}

/**
 * Middleware that gives every request a session: an existing one when the
 * request carries a session cookie, a new one otherwise (with the cookie
 * set on the response). The session id is put into the request extensions
 * for handlers to pick up.
 */
pub struct SessionMiddleware;

impl<S, B> Transform<S, ServiceRequest> for SessionMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SessionMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SessionMiddlewareService { service }))
    }
}

pub struct SessionMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SessionMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let set_cookie = req
            .app_data::<Data<AppState>>()
            .cloned()
            .map(|state| {
                let cookie = req.cookie(SESSION_COOKIE).map(|c| c.value().to_string());
                let (id, is_new) = state.sessions.resolve(cookie.as_deref());
                req.extensions_mut().insert(SessionId(id));
                is_new
            })
            .unwrap_or(false);

        let future = self.service.call(req);
        Box::pin(async move {
            let mut response = future.await?;
            if set_cookie {
                // The cookie name and value are generated here and always
                // valid, so the parse error cannot happen.
                let id = response
                    .request()
                    .extensions()
                    .get::<SessionId>()
                    .expect("session middleware inserted the id")
                    .0
                    .clone();
                let _ = response.response_mut().add_cookie(&session_cookie(&id));
            }
            Ok(response)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_id_is_32_hex_characters() {
        let id = new_id();
        assert!(is_valid_id(&id), "unexpected id: {id}");
    }

    #[test]
    fn test_ids_are_unique() {
        assert_ne!(new_id(), new_id());
    }

    #[test]
    fn test_is_valid_id_rejects_other_strings() {
        assert!(!is_valid_id(""));
        assert!(!is_valid_id("short"));
        assert!(!is_valid_id("gggggggggggggggggggggggggggggggg"));
        assert!(!is_valid_id("000000000000000000000000000000000")); // 33 chars
    }

    #[test]
    fn test_resolve_without_a_cookie_creates_a_new_session() {
        let store = SessionStore::new();

        let (id, is_new) = store.resolve(None);

        assert!(is_new);
        assert!(is_valid_id(&id));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_resolve_with_a_session_cookie_reuses_the_session() {
        let store = SessionStore::new();
        let (id, _) = store.resolve(None);

        let (resolved, is_new) = store.resolve(Some(&id));

        assert!(!is_new);
        assert_eq!(resolved, id);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_resolve_with_an_invalid_cookie_creates_a_new_session() {
        let store = SessionStore::new();

        let (id, is_new) = store.resolve(Some("not-a-session-id"));

        assert!(is_new);
        assert!(is_valid_id(&id));
        // The invalid cookie did not create a session of its own.
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_resolve_adopts_an_unknown_but_valid_cookie() {
        let store = SessionStore::new();
        let id = new_id();

        let (resolved, is_new) = store.resolve(Some(&id));

        assert!(!is_new);
        assert_eq!(resolved, id);
        // The session came back to life (e.g. after a server restart).
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_sweep_older_than_drops_idle_sessions() {
        let store = SessionStore::new();
        store.resolve(None);
        store.resolve(None);

        // Nothing is older than a long max age.
        store.sweep_older_than(Duration::from_secs(60 * 60));
        assert_eq!(store.len(), 2);

        // ... but a max age of zero (elapsed is never below it) drops
        // every session.
        store.sweep_older_than(Duration::ZERO);
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_sweep_uses_last_seen_not_creation() {
        let store = SessionStore::new();
        let (kept, _) = store.resolve(None);
        store.resolve(None);

        // Backdate one session past the max age.
        {
            let mut sessions = store.sessions.write().unwrap();
            sessions.get_mut(&kept).unwrap().last_seen = Instant::now() - Duration::from_secs(120);
        }

        // The backdated session is dropped, the recently-seen one survives.
        store.sweep_older_than(Duration::from_secs(60));
        assert_eq!(store.len(), 1);

        // Seeing a session again revives it: it now survives the same sweep.
        store.resolve(Some(&kept));
        store.sweep_older_than(Duration::from_secs(60));
        assert_eq!(store.len(), 2);
    }
}
