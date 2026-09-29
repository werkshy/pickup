use actix_web::App;
use utoipa::OpenApi;
use utoipa_actix_web::AppExt;

/**
 * Base OpenAPI document. Paths and schemas are not listed here: they are
 * collected automatically from the services registered in
 * `api::register_services`, which requires every handler to be annotated
 * with `#[utoipa::path]`.
 */
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Pickup API",
        version = env!("CARGO_PKG_VERSION"),
        description = "API for the Pickup music player server.",
    ),
    tags(
        (name = "index", description = "Server info."),
        (name = "control", description = "Playback control: play, stop, volume."),
        (name = "collection", description = "Browsing the music collection."),
        (name = "queue", description = "The playback queue."),
    )
)]
pub struct ApiDoc;

/**
 * The base document that service documentation is collected into.
 */
pub fn base_doc() -> utoipa::openapi::OpenApi {
    let mut doc = ApiDoc::openapi();
    // Cargo has no license set, don't emit an empty license object in the spec
    doc.info.license = None;
    doc
}

/**
 * The complete OpenAPI spec. Registers the API services on a bare app
 * (without state, so nothing runs) just to collect their documentation.
 */
pub fn spec() -> utoipa::openapi::OpenApi {
    super::register_services(App::new().into_utoipa_app().openapi(base_doc()))
        .split_for_parts()
        .1
}
