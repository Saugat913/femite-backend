use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, ToSchema)]
pub struct OrderResponse {
    pub id: Uuid,
    #[schema(value_type = String, example = "123.45")]
    pub total: Decimal,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateOrderRequest {
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateOrderResponse {
    pub id: Uuid,
    #[schema(value_type = String, example = "123.45")]
    pub total: Decimal,
    pub status: String,
    pub items_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrderItemResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub product_name: String,
    pub product_image_url: Option<String>,
    pub quantity: i32,
    #[schema(value_type = String, example = "9.99")]
    pub price: Decimal,
    #[schema(value_type = String, example = "19.98")]
    pub subtotal: Decimal,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrderDetailsResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    #[schema(value_type = String, example = "123.45")]
    pub total: Decimal,
    pub status: String,
    pub payment_id: Option<Uuid>,
    pub items: Vec<OrderItemResponse>,
    pub created_at: DateTime<Utc>,
}
