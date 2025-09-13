mod common;

use axum_test::TestServer;
use serde_json::json;

#[tokio::test]
async fn auth_protected_endpoints_require_token() {
    let server = common::test_server_lazy().await;

    // Inventory admin-only endpoints
    server.get("/api/inventory/alerts").await.assert_status_unauthorized();
    server.get("/api/inventory/report").await.assert_status_unauthorized();

    // Order endpoints needing auth
    server.get("/api/order/my").await.assert_status_unauthorized();
    server.get("/api/order/all").await.assert_status_unauthorized();

    // Payment endpoints needing auth
    server
        .post("/api/payment/create-payment-intent")
        .json(&json!({
            "amount": "10.00",
            "currency": "usd",
            "order_id": "00000000-0000-0000-0000-000000000000"
        }))
        .await
        .assert_status_unauthorized();
}

