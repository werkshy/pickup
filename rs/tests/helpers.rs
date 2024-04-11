use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    App, Error,
};
use pickup::{build_app, build_app_state, filemanager::options::CollectionOptions, ServeOptions};

pub fn build_test_app() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
> {
    let options = ServeOptions {
        collection_options: CollectionOptions {
            dir: String::from("../music"),
            ignores: None,
        },
        port: 3001,
    };

    let app_state = build_app_state(&options);
    return build_app(app_state);
}
