use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    #[schema(value_type = String, example = "123.45")]
    pub total: Decimal,
    pub status: String,
    pub payment_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    #[schema(value_type = String, example = "9.99")]
    pub price: Decimal,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct OrderWithItems {
    #[sqlx(flatten)]
    pub order: Order,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateStatusDto {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Cart,
    PendingPayment,
    PaymentProcessing,
    Paid,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Refunded,
}

impl ToString for OrderStatus {
    fn to_string(&self) -> String {
        match self {
            OrderStatus::Cart => "cart".to_string(),
            OrderStatus::PendingPayment => "pending_payment".to_string(),
            OrderStatus::PaymentProcessing => "payment_processing".to_string(),
            OrderStatus::Paid => "paid".to_string(),
            OrderStatus::Processing => "processing".to_string(),
            OrderStatus::Shipped => "shipped".to_string(),
            OrderStatus::Delivered => "delivered".to_string(),
            OrderStatus::Cancelled => "cancelled".to_string(),
            OrderStatus::Refunded => "refunded".to_string(),
        }
    }
}
