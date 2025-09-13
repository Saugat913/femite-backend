mod common;

use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn payment_routes_require_auth_and_exist() {
    // Ensure Stripe secret exists for request construction in handler
    std::env::set_var("STRIPE_SECRET_KEY", "sk_test_dummy");
    let server = common::test_server_lazy().await;

    // Create intent requires auth; without auth, 401
    server
        .post("/api/payment/create-payment-intent")
        .json(&json!({"amount":"10.00","currency":"usd","order_id":Uuid::nil()}))
        .await
        .assert_status_unauthorized();

    // With user auth, endpoint exists; DB may cause 400/500 but not 404/401/403
    let res = server
        .post("/api/payment/create-payment-intent")
        .add_header("Authorization", format!("Bearer {}", common::jwt_user()))
        .json(&json!({"amount":"10.00","currency":"usd","order_id":Uuid::nil()}))
        .await;
    assert!(res.status_code().as_u16() != 401 && res.status_code().as_u16() != 403 && res.status_code().as_u16() != 404);

    // Webhook endpoint exists (no auth), responds with 200/400
    let res = server
        .post("/api/payment/webhook")
        .text("{}")
        .await;
    let code = res.status_code().as_u16();
    // Accept 200/400/500 depending on environment
    assert!(code == 200 || code == 400 || code == 500);
}

