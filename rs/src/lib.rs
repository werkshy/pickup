pub mod api;
pub mod app_state;
pub mod error;
pub mod filemanager;
pub mod player;
pub mod queue;

use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::thread;

use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceResponse},
    error::{JsonPayloadError, PathError},
    http::StatusCode,
    web::{self, Data},
    Error,
};
use actix_web::{dev::ServiceRequest, middleware::Logger, ResponseError};
use actix_web::{App, HttpResponse, HttpServer};

use crate::api::types::ApiErrorCode;
use app_state::AppState;
use error::AppError;
use filemanager::collection;
use filemanager::options::CollectionOptions;
use player::{Player, PlayerClient};
use utoipa_actix_web::AppExt;

// Enable assert_matches in tests
#[cfg(test)]
#[macro_use]
extern crate assert_matches;

#[derive(Clone)]
pub struct ServeOptions {
    pub collection_options: CollectionOptions,
    pub port: u32,
}

/**
 * Create the app state we need to pass into the app.
 */
pub fn build_app_state(options: &ServeOptions) -> AppState {
    let player = spawn_player();
    let collection = collection::init(options.collection_options.clone()).unwrap();
    let collection_arc = Arc::new(collection);
    let queue = queue::PlaybackQueue::new();
    AppState {
        player,
        collection: collection_arc,
        queue: RwLock::new(queue),
    }
}

/**
 * It's not well documented how to return an App from a function, but there is a test that shows how:
 * https://github.com/actix/actix-web/blob/b1c85ba85be91b5ea34f31264853b411fadce1ef/actix-web/src/app.rs#L698
 */
pub fn build_app(
    app_state: Data<AppState>,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
> {
    api::register_services(
        App::new()
            .into_utoipa_app()
            .openapi(api::openapi::base_doc())
            .app_data(app_state)
            // Keep JSON parse/validation failures in the same shape as every other error
            .app_data(web::JsonConfig::default().error_handler(|error, _req| {
                AppError::new(
                    ApiErrorCode::InvalidRequest,
                    json_error_message(&error),
                    StatusCode::BAD_REQUEST,
                )
                .into()
            }))
            // Keep path parameter extraction failures in the same shape too
            .app_data(web::PathConfig::default().error_handler(|error, _req| {
                AppError::new(
                    ApiErrorCode::InvalidRequest,
                    path_error_message(&error),
                    StatusCode::BAD_REQUEST,
                )
                .into()
            }))
            .map(|app| app.wrap(Logger::default()))
            .default_service(web::to(not_found)),
    )
    // Serve the spec collected from the services above; must come after them
    .openapi_service(|spec| {
        web::resource("/openapi.json")
            .route(web::get().to(move || {
                let spec = spec.clone();
                async move { web::Json(spec) }
            }))
            // Route level method guard, so a wrong method lands on this
            // resource's fallback instead of the app-wide one
            .default_service(web::to(not_found))
    })
    .into_app()
}

/**
 * Actix's own `JsonPayloadError` messages are written for framework
 * debugging, e.g. "Json deserialize error: missing field `category` at line 1
 * column 11". Map each case onto wording that means something to an API
 * consumer, and so doesn't change when the dependency is upgraded.
 */
fn json_error_message(error: &JsonPayloadError) -> String {
    match error {
        JsonPayloadError::ContentType => {
            "Expected a JSON body with Content-Type: application/json".to_string()
        }
        JsonPayloadError::Deserialize(error) => format!("Invalid JSON body: {}", error),
        JsonPayloadError::Overflow { limit } => {
            format!("JSON body is over the {} byte limit", limit)
        }
        JsonPayloadError::OverflowKnownLength { length, limit } => {
            format!(
                "JSON body is {} bytes, over the {} byte limit",
                length, limit
            )
        }
        JsonPayloadError::Payload(error) => {
            format!("Could not read the request body: {}", error)
        }
        JsonPayloadError::Serialize(error) => {
            format!("Could not serialize the response: {}", error)
        }
        // `JsonPayloadError` is `#[non_exhaustive]`, so actix may add cases
        _ => "Malformed request body".to_string(),
    }
}

/**
 * Fallback for requests no route claimed: unknown paths, and methods that a
 * matched path does not serve. Both come back as 404 in the standard shape.
 */
async fn not_found() -> HttpResponse {
    AppError::not_found("No such route").error_response()
}

/**
 * Path extraction errors are also written for framework debugging, e.g.
 * "Path deserialize error: invalid digit found in string". Say what the API
 * consumer did wrong instead.
 */
fn path_error_message(error: &PathError) -> String {
    match error {
        PathError::Deserialize(error) => format!("Invalid path parameter: {}", error),
        // `PathError` is `#[non_exhaustive]`, so actix may add cases
        _ => "Invalid path parameter".to_string(),
    }
}

pub async fn serve(options: ServeOptions) -> std::io::Result<()> {
    let address = format!("0.0.0.0:{}", options.port);
    let app_state = Data::new(build_app_state(&options));
    log::info!("Starting on http://{}", address);
    HttpServer::new(move || build_app(app_state.clone()))
        .workers(2)
        .bind(address.as_str())?
        .shutdown_timeout(60) // <- Set shutdown timeout to 60 seconds
        .run()
        .await
}

/**
 * Spawn a new thread with a Player instance (rodio's Sink can't be shared
 * across threads). Create a channel to send commands into the Player thread,
 * and return a PlayerClient (internal interface for sending commands.)
 */
pub fn spawn_player() -> PlayerClient {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut player = Player::new();

        for command in rx {
            player.command(command);
        }
    });
    PlayerClient::new(tx)
}
