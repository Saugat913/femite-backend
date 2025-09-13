mod common;

use axum_test::TestServer;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    email: String,
    role: String,
    exp: usize,
}

fn jwt(user_id: Uuid, role: &str) -> String {
    let claims = Claims {
        sub: user_id,
        email: "user@example.com".to_string(),
        role: role.to_string(),
        exp: 4102444800, // year 2100
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(b"test_secret")).unwrap()
}

#[tokio::test]
async fn open_product_routes() {
    let server = common::test_server_lazy().await;

    let res = server.get("/api/product").await;
    let status = res.status_code().as_u16();
    // Endpoint is public, so it should not be 401/403 even if DB is unavailable
    assert!(status != 401 && status != 403);
}

#[tokio::test]
async fn create_product_requires_admin() {
    let server = common::test_server_lazy().await;

    let token = jwt(Uuid::new_v4(), "user");
    server
        .post("/api/product")
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "name": "Test Product",
            "description": "Desc",
            "price": "9.99",
            "stock": 10,
            "image_url": null,
            "low_stock_threshold": 2,
            "track_inventory": true
        }))
        .await
        .assert_status_unauthorized();
}

#[tokio::test]
async fn admin_protected_inventory_endpoints_check() {
    let server = common::test_server_lazy().await;

    let token = jwt(Uuid::new_v4(), "admin");

    // Will likely fail with 500/404 due to missing DB; but should not be 401/403
    let res = server
        .get("/api/inventory/alerts")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;
    assert!(res.status_code().as_u16() != 401 && res.status_code().as_u16() != 403);
}

