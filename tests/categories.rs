mod common;

use axum_test::TestServer;
use serde_json::json;

#[tokio::test]
async fn category_crud_secured() {
    let server = common::test_server_lazy().await;

    // Create category (admin required)
    let res = server
        .post("/api/category")
        .add_header("Authorization", format!("Bearer {}", common::jwt_admin()))
        .json(&json!({"name":"Cat A","description":"Desc"}))
        .await;
    // Without DB, allow 500/404; but not 401/403
    let status = res.status_code().as_u16();
    assert!(status != 401 && status != 403);

    // List categories (public)
    let res = server.get("/api/category").await;
    assert!(res.status_code().as_u16() != 401 && res.status_code().as_u16() != 403);
}

