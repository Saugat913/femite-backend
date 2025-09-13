use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct StockReservation {
    pub id: Uuid,
    pub product_id: Uuid,
    pub cart_id: Uuid,
    pub quantity: i32,
    pub reserved_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct InventoryLog {
    pub id: Uuid,
    pub product_id: Uuid,
    pub change_type: String,
    pub quantity_change: i32,
    pub previous_stock: i32,
    pub new_stock: i32,
    pub reference_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum InventoryChangeType {
    StockIn,
    StockOut,
    Reserved,
    Unreserved,
    Sold,
}

impl ToString for InventoryChangeType {
    fn to_string(&self) -> String {
        match self {
            InventoryChangeType::StockIn => "stock_in".to_string(),
            InventoryChangeType::StockOut => "stock_out".to_string(),
            InventoryChangeType::Reserved => "reserved".to_string(),
            InventoryChangeType::Unreserved => "unreserved".to_string(),
            InventoryChangeType::Sold => "sold".to_string(),
        }
    }
}

impl InventoryChangeType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "stock_in" => Some(InventoryChangeType::StockIn),
            "stock_out" => Some(InventoryChangeType::StockOut),
            "reserved" => Some(InventoryChangeType::Reserved),
            "unreserved" => Some(InventoryChangeType::Unreserved),
            "sold" => Some(InventoryChangeType::Sold),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StockUpdateRequest {
    pub quantity: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StockReservationRequest {
    pub product_id: Uuid,
    pub quantity: i32,
    pub expires_in_minutes: Option<i32>, // defaults to 30 minutes
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LowStockAlert {
    pub product_id: Uuid,
    pub product_name: String,
    pub current_stock: i32,
    pub available_stock: i32,
    pub threshold: i32,
    pub is_critical: bool, // true if stock is 0 or negative
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InventoryReport {
    pub total_products: i32,
    pub low_stock_products: i32,
    pub out_of_stock_products: i32,
    pub total_reserved: i32,
    pub total_available: i32,
    pub alerts: Vec<LowStockAlert>,
}
