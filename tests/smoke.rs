mod common;

use axum_test::TestServer;

#[tokio::test]
async fn health_ok() {
    let server = common::test_server_lazy().await;
    let res = server.get("/health").await;
    res.assert_status_ok();
    res.assert_text("OK");
}

#[tokio::test]
async fn openapi_json_ok() {
    let server = common::test_server_lazy().await;
    let res = server.get("/api-docs/openapi.json").await;
    res.assert_status_ok();
    // Minimal structural validation
    let json = res.json::<serde_json::Value>();
    assert!(json.get("openapi").is_some());
    assert!(json.get("paths").is_some());
}

