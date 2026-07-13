#![cfg(test)]

mod fixtures;

use crate::fixtures::{TestServerHandles, get_token_review_endpoint, internal_token};
use anyhow::Result;
use boxer_core::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use boxer_core::services::audit::chained::audit_event::AuditEvent;
use fixtures::{with_logging, with_test_server};
use mockall::mock;
use reqwest::Client;
use rstest::rstest;
use std::time::Duration;

#[rstest]
#[timeout(Duration::from_secs(15))]
#[actix_web::test]
async fn test_internal_token_issuance(
    _with_logging: (),
    #[future] with_test_server: TestServerHandles,
    #[future] internal_token: Result<String>,
) -> () {
    // Arrange

    let (server_handle, thread_handle, server_address) = with_test_server.await;
    let internal_token = internal_token.await.expect("failed to get internal_token");

    // Act
    let validation_result = Client::new()
        .get(get_token_review_endpoint(server_address))
        .header("X-Original-Url", "http://example.com/api/v1/example/")
        .header("X-Original-Method", "GET")
        .bearer_auth(internal_token)
        .send()
        .await
        .expect("Failed to call token review endpoint");

    // Assert
    println!("Token review endpoint returned: {:#?}", validation_result);
    assert_eq!(validation_result.status(), 200,);

    // Cleanup
    server_handle.stop(true).await;
    thread_handle.await.unwrap().expect("Failed to join server thread");
}

mock! {
    pub AuditWriter {}

    impl AuditWriter for AuditWriter {
        fn write(&self, event: AuditEvent);
    }

}
