use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web::Data,
    App, Error,
};
use pickup::{
    app_state::AppState, build_app, build_app_state, filemanager::options::CollectionOptions,
    ServeOptions,
};

#[allow(dead_code)] // Used in tests, but triggers dead_code on test binaries that don't use it.
pub fn build_test_app() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
> {
    build_app(build_test_app_state())
}

pub fn build_test_app_state() -> Data<AppState> {
    let options = ServeOptions {
        collection_options: CollectionOptions {
            dir: String::from("../music"),
            ignores: None,
        },
        port: 3001,
    };

    Data::new(build_app_state(&options))
}
