mod common;

use axum_test::TestServer;

#[tokio::test]
async fn image_upload_requires_admin() {
    let server = common::test_server_lazy().await;

    // Without admin token expect 401
    server.post("/api/image/upload").await.assert_status_unauthorized();

    // With admin token, endpoint exists; without Cloudinary creds this may 500
    let res = server
        .post("/api/image/upload")
        .add_header("Authorization", format!("Bearer {}", common::jwt_admin()))
        .await;
    assert!(res.status_code().as_u16() != 401 && res.status_code().as_u16() != 403);
}

