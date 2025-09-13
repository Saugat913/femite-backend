use hemp_backend::{
    dtos::{NewProductDto, ProductResponse, UpdateProductDto, SignupDto, LoginDto, Claims},
    routes::build_route,
    state::AppState,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use rust_decimal::Decimal;
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{collections::HashMap, str::from_utf8, sync::Arc};
use tower::ServiceExt;
use uuid::Uuid;
use jsonwebtoken::{encode, Header, EncodingKey};
use std::time::{SystemTime, UNIX_EPOCH};

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/hemp_backend_test".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await.expect("Failed to run migrations");
    
    pool
}

async fn setup_test_app() -> Router {
    let pool = setup_test_db().await;
    
    let state = AppState {
        db: pool.clone(),
        jwt_secret: Arc::new("test_secret".to_string()),
        cloudinary_cloud_name: Arc::new("test_cloud".to_string()),
        cloudinary_api_key: Arc::new("test_key".to_string()),
        cloudinary_api_secret: Arc::new("test_secret".to_string()),
    };
    
    build_route(state)
}

async fn cleanup_test_data(pool: &PgPool) {
    sqlx::query("DELETE FROM order_items").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM orders").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM cart_items").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM carts").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM stock_reservations").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM inventory_logs").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM product_categories").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM products").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM categories").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM users").execute(pool).await.unwrap();
}

fn create_admin_token() -> String {
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + 3600; // 1 hour from now
    
    let claims = Claims {
        sub: Uuid::new_v4(),
        email: "admin@test.com".to_string(),
        role: "admin".to_string(),
        exp: exp as usize,
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("test_secret".as_ref()),
    ).unwrap()
}

// Helper function to create a test product
fn create_test_product_dto() -> NewProductDto {
    NewProductDto {
        name: "Test Hemp Oil".to_string(),
        description: Some("High quality CBD oil".to_string()),
        price: Decimal::new(2999, 2), // $29.99
        stock: 100,
        image_url: Some("https://example.com/image.jpg".to_string()),
        low_stock_threshold: Some(10),
        track_inventory: Some(true),
    }
}

#[tokio::test]
#[ignore = "Requires TEST_DATABASE_URL and Postgres running"]
async fn test_create_product_success() {
    let app = setup_test_app().await;
    let admin_token = create_admin_token();
    
    let new_product = create_test_product_dto();
    let request_body = serde_json::to_string(&new_product).unwrap();
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/product")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", admin_token))
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let product: ProductResponse = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(product.name, new_product.name);
    assert_eq!(product.price, new_product.price);
    assert_eq!(product.stock, new_product.stock);
    assert_eq!(product.image_url, new_product.image_url);
    
    // cleanup skipped in this simplified test harness
}

#[tokio::test]
#[ignore = "Requires TEST_DATABASE_URL and Postgres running"]
async fn test_get_product_success() {
    let app = setup_test_app().await;
    let admin_token = create_admin_token();
    
    // First create a product
    let new_product = create_test_product_dto();
    let request_body = serde_json::to_string(&new_product).unwrap();
    
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/product")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", admin_token))
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();
    
    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX).await.unwrap();
    let created_product: ProductResponse = serde_json::from_slice(&create_body).unwrap();
    
    // Now get the product
    let get_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/product/{}", created_product.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(get_response.status(), StatusCode::OK);
    
    let get_body = axum::body::to_bytes(get_response.into_body(), usize::MAX).await.unwrap();
    let retrieved_product: ProductResponse = serde_json::from_slice(&get_body).unwrap();
    
    assert_eq!(retrieved_product.id, created_product.id);
    assert_eq!(retrieved_product.name, new_product.name);
}

#[tokio::test]
#[ignore = "Requires TEST_DATABASE_URL and Postgres running"]
async fn test_get_product_not_found() {
    let app = setup_test_app().await;
    let non_existent_id = Uuid::new_v4();
    
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/product/{}", non_existent_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "Requires TEST_DATABASE_URL and Postgres running"]
async fn test_list_products_success() {
    let app = setup_test_app().await;
    
    // Create multiple products
    for i in 0..3 {
        let mut product = create_test_product_dto();
        product.name = format!("Test Product {}", i);
        let request_body = serde_json::to_string(&product).unwrap();
        
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/product")
                    .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", create_admin_token()))
                    .body(Body::from(request_body))
                    .unwrap(),
            )
            .await
            .unwrap();
    }
    
    // List products
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/product")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let products: Vec<ProductResponse> = serde_json::from_slice(&body).unwrap();
    
    assert!(products.len() >= 3);
}

#[tokio::test]
#[ignore = "Requires TEST_DATABASE_URL and Postgres running"]
async fn test_update_product_success() {
    let app = setup_test_app().await;
    
    // First create a product
    let new_product = create_test_product_dto();
    let request_body = serde_json::to_string(&new_product).unwrap();
    
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/product")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", create_admin_token()))
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();
    
    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX).await.unwrap();
    let created_product: ProductResponse = serde_json::from_slice(&create_body).unwrap();
    
    // Update the product
    let update_dto = UpdateProductDto {
        name: Some("Updated Hemp Oil".to_string()),
        description: Some("Updated description".to_string()),
        price: Some(Decimal::new(3999, 2)), // $39.99
        stock: Some(50),
        image_url: Some("https://example.com/new_image.jpg".to_string()),
        low_stock_threshold: Some(5),
        track_inventory: Some(false),
    };
    
    let update_body = serde_json::to_string(&update_dto).unwrap();
    
    let update_response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/api/product/{}", created_product.id))
                .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", create_admin_token()))
                .body(Body::from(update_body))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(update_response.status(), StatusCode::OK);
    
    let update_response_body = axum::body::to_bytes(update_response.into_body(), usize::MAX).await.unwrap();
    let updated_product: ProductResponse = serde_json::from_slice(&update_response_body).unwrap();
    
    assert_eq!(updated_product.name, "Updated Hemp Oil");
    assert_eq!(updated_product.price, Decimal::new(3999, 2));
    assert_eq!(updated_product.stock, 50);
}

#[tokio::test]
#[ignore = "Requires TEST_DATABASE_URL and Postgres running"]
async fn test_delete_product_success() {
    let app = setup_test_app().await;
    
    // First create a product
    let new_product = create_test_product_dto();
    let request_body = serde_json::to_string(&new_product).unwrap();
    
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/product")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", create_admin_token()))
                .body(Body::from(request_body))
                .unwrap(),
        )
        .await
        .unwrap();
    
    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX).await.unwrap();
    let created_product: ProductResponse = serde_json::from_slice(&create_body).unwrap();
    
    // Delete the product
    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/product/{}", created_product.id))
                .header("authorization", format!("Bearer {}", create_admin_token()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
    
    // Verify product is deleted
    let get_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/product/{}", created_product.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "Requires TEST_DATABASE_URL and Postgres running"]
async fn test_health_endpoint() {
    let app = setup_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = from_utf8(&body).unwrap();
    
    assert_eq!(body_str, "OK");
}

// Note: Image upload tests would require mocking Cloudinary or setting up test credentials
// For now, we'll focus on the core product functionality tests
