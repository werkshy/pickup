mod index;
pub use index::hello;

pub mod control;
pub mod list;
pub mod openapi;
pub mod queue;
pub mod types;

use actix_web::dev::{ServiceFactory, ServiceRequest};
use actix_web::Error;
use utoipa_actix_web::UtoipaApp;

/**
 * The single list of API services. Registering through `UtoipaApp` both
 * routes the app and feeds the OpenAPI spec, so a handler that is not in
 * this list is neither served nor documented. Handlers must be annotated
 * with `#[utoipa::path]` to be documented.
 */
pub fn register_services<T>(app: UtoipaApp<T>) -> UtoipaApp<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    app.service(hello)
        .service(control::play)
        .service(control::stop)
        .service(control::volume)
        .service(control::get_volume)
        .service(list::list_categories)
        .service(queue::add)
        .service(queue::clear)
        .service(queue::get_queue)
}
