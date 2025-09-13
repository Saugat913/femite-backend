mod common;

use axum_test::TestServer;
use serde_json::json;

#[tokio::test]
async fn inventory_stock_public_and_admin_paths() {
    let server = common::test_server_lazy().await;

    // Public stock query (product_id random)
    let product_id = uuid::Uuid::new_v4();
    let res = server.get(&format!("/api/inventory/products/{}/stock", product_id)).await;
    // Could be 404/500 without DB, but not 401/403
    assert!(res.status_code().as_u16() != 401 && res.status_code().as_u16() != 403);

    // Admin update stock
    let res = server
        .put(&format!("/api/inventory/products/{}/stock", product_id))
        .add_header("Authorization", format!("Bearer {}", common::jwt_admin()))
        .json(&json!({"quantity": 5, "notes": "test"}))
        .await;
    assert!(res.status_code().as_u16() != 401 && res.status_code().as_u16() != 403);
}

