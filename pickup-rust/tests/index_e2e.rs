use actix_web::test;

use crate::helpers::build_test_app;

mod helpers;

#[actix_web::test]
async fn test_index_get() {
    let app = test::init_service(build_test_app()).await;
    let req = test::TestRequest::get().uri("/").to_request();

    let resp = test::call_and_read_body(&app, req).await;

    assert_eq!(&resp[..], b"Hello");
}
