use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[schema(value_type = String, example = "19.99")]
    pub price: Decimal,
    pub stock: i32,
    pub low_stock_threshold: Option<i32>,
    pub track_inventory: bool,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct ProductWithAvailableStock {
    #[sqlx(flatten)]
    pub product: Product,
    pub available_stock: i32, // stock minus reserved quantities
    pub is_low_stock: bool,
}
