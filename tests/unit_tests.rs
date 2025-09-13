use hemp_backend::{
    dtos::{NewProductDto, UpdateProductDto},
    model::product::Product,
    repository::ProductRepository,
    services::product_service::ProductService,
};
use rust_decimal::Decimal;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

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

#[tokio::test]
#[ignore = "Requires TEST_DATABASE_URL and Postgres running"]
async fn test_product_repository_create() {
    let pool = setup_test_db().await;
    let repo = ProductRepository::new(pool);
    
    let result = repo.create(
        "Test Product",
        Some("Test description"),
        Decimal::new(1999, 2), // $19.99
        50,
        Some("https://example.com/image.jpg"),
        Some(10),
        true,
    ).await;
    
    assert!(result.is_ok());
    let product = result.unwrap();
    
    assert_eq!(product.name, "Test Product");
    assert_eq!(product.description, Some("Test description".to_string()));
    assert_eq!(product.price, Decimal::new(1999, 2));
    assert_eq!(product.stock, 50);
    assert_eq!(product.image_url, Some("https://example.com/image.jpg".to_string()));
    assert_eq!(product.low_stock_threshold, Some(10));
    assert_eq!(product.track_inventory, true);
}

#[tokio::test]
#[ignore = "Requires TEST_DATABASE_URL and Postgres running"]
async fn test_product_repository_get() {
    let pool = setup_test_db().await;
    let repo = ProductRepository::new(pool);
    
    // Create a product first
    let created = repo.create(
        "Get Test Product",
        Some("Get test description"),
        Decimal::new(2499, 2),
        25,
        None,
        None,
        true,
    ).await.unwrap();
    
    // Get the product
    let result = repo.get(created.id).await;
    assert!(result.is_ok());
    
    let retrieved = result.unwrap();
    assert!(retrieved.is_some());
    
    let product = retrieved.unwrap();
    assert_eq!(product.id, created.id);
    assert_eq!(product.name, "Get Test Product");
}

#[tokio::test]
#[ignore = "Requires TEST_DATABASE_URL and Postgres running"]
async fn test_product_repository_update() {
    let pool = setup_test_db().await;
    let repo = ProductRepository::new(pool);
    
    // Create a product first
    let created = repo.create(
        "Update Test Product",
        Some("Original description"),
        Decimal::new(1500, 2),
        100,
        None,
        Some(20),
        true,
    ).await.unwrap();
    
    // Update the product
    let result = repo.update(
        created.id,
        Some("Updated Test Product"),
        Some("Updated description"),
        Some(Decimal::new(1750, 2)),
        Some(75),
        Some("https://example.com/updated_image.jpg"),
        Some(15),
        Some(false),
    ).await;
    
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert!(updated.is_some());
    
    let product = updated.unwrap();
    assert_eq!(product.name, "Updated Test Product");
    assert_eq!(product.description, Some("Updated description".to_string()));
    assert_eq!(product.price, Decimal::new(1750, 2));
    assert_eq!(product.stock, 75);
    assert_eq!(product.image_url, Some("https://example.com/updated_image.jpg".to_string()));
    assert_eq!(product.low_stock_threshold, Some(15));
    assert_eq!(product.track_inventory, false);
}

#[tokio::test]
#[ignore = "Requires TEST_DATABASE_URL and Postgres running"]
async fn test_product_service_create() {
    let pool = setup_test_db().await;
    let repo = ProductRepository::new(pool);
    let service = ProductService::new(repo);
    
    let dto = NewProductDto {
        name: "Service Test Product".to_string(),
        description: Some("Service test description".to_string()),
        price: Decimal::new(3500, 2), // $35.00
        stock: 200,
        image_url: Some("https://example.com/service_image.jpg".to_string()),
        low_stock_threshold: Some(25),
        track_inventory: Some(true),
    };
    
    let result = service.create(dto).await;
    assert!(result.is_ok());
    
    let product = result.unwrap();
    assert_eq!(product.name, "Service Test Product");
    assert_eq!(product.price, Decimal::new(3500, 2));
    assert_eq!(product.stock, 200);
}

#[tokio::test]
#[ignore = "Requires TEST_DATABASE_URL and Postgres running"]
async fn test_product_service_list() {
    let pool = setup_test_db().await;
    let repo = ProductRepository::new(pool.clone());
    let service = ProductService::new(repo);
    
    // Clean existing products for this test
    sqlx::query("DELETE FROM products WHERE name LIKE 'List Test Product%'")
        .execute(&pool)
        .await
        .unwrap();
    
    // Create multiple products
    for i in 0..5 {
        let dto = NewProductDto {
            name: format!("List Test Product {}", i),
            description: Some(format!("List test description {}", i)),
            price: Decimal::new(1000 + ((i as i64) * 100), 2),
            stock: 10 + i as i32,
            image_url: None,
            low_stock_threshold: Some(5),
            track_inventory: Some(true),
        };
        
        service.create(dto).await.unwrap();
    }
    
    // List products
    let result = service.list(10, 0).await;
    assert!(result.is_ok());
    
    let products = result.unwrap();
    // Should have at least the 5 products we just created
    assert!(products.len() >= 5);
    
    // Check that our test products are there
    let test_products: Vec<&Product> = products
        .iter()
        .filter(|p| p.name.starts_with("List Test Product"))
        .collect();
    assert_eq!(test_products.len(), 5);
}

#[tokio::test]
async fn test_decimal_precision() {
    // Test that we handle decimal prices correctly
    let price = Decimal::new(2999, 2); // $29.99
    assert_eq!(price.to_string(), "29.99");
    
    let price2 = Decimal::new(100, 2); // $1.00
    assert_eq!(price2.to_string(), "1.00");
    
    let price3 = Decimal::new(50, 2); // $0.50
    assert_eq!(price3.to_string(), "0.50");
}
