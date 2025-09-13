use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;
use validator::Validate;
use utoipa::ToSchema;
use crate::model::product::Product;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct NewProductDto {
    #[validate(length(min = 1, max = 255, message = "Product name must be between 1 and 255 characters"))]
    pub name: String,
    #[validate(length(max = 1000, message = "Description must not exceed 1000 characters"))]
    pub description: Option<String>,
    // #[validate(range(min = 0.01, message = "Price must be greater than 0"))] // Temporarily disabled
    #[schema(value_type = String, example = "123.45")]
    pub price: Decimal,
    #[validate(range(min = 0, message = "Stock cannot be negative"))]
    pub stock: i32,
    #[validate(url(message = "Invalid image URL format"))]
    pub image_url: Option<String>,
    #[validate(range(min = 0, message = "Low stock threshold cannot be negative"))]
    pub low_stock_threshold: Option<i32>,
    pub track_inventory: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateProductDto {
    #[validate(length(min = 1, max = 255, message = "Product name must be between 1 and 255 characters"))]
    pub name: Option<String>,
    #[validate(length(max = 1000, message = "Description must not exceed 1000 characters"))]
    pub description: Option<String>,
    // #[validate(range(min = 0.01, message = "Price must be greater than 0"))] // Temporarily disabled
    #[schema(value_type = String, example = "123.45")]
    pub price: Option<Decimal>,
    #[validate(range(min = 0, message = "Stock cannot be negative"))]
    pub stock: Option<i32>,
    #[validate(url(message = "Invalid image URL format"))]
    pub image_url: Option<String>,
    #[validate(range(min = 0, message = "Low stock threshold cannot be negative"))]
    pub low_stock_threshold: Option<i32>,
    pub track_inventory: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProductResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[schema(value_type = String, example = "123.45")]
    pub price: Decimal,
    pub stock: i32,
    pub image_url: Option<String>,
    pub low_stock_threshold: Option<i32>,
    pub track_inventory: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<Product> for ProductResponse {
    fn from(p: Product) -> Self {
        ProductResponse {
            id: p.id,
            name: p.name,
            description: p.description,
            price: p.price,
            stock: p.stock,
            image_url: p.image_url,
            low_stock_threshold: p.low_stock_threshold,
            track_inventory: p.track_inventory,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}
