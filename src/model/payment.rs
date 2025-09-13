use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Payment {
    pub id: Uuid,
    pub order_id: Uuid,
    pub stripe_payment_intent_id: String,
    #[schema(value_type = String, example = "123.45")]
    pub amount: BigDecimal,
    pub currency: String,
    pub status: String,
    pub payment_method: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct PaymentWebhook {
    pub id: Uuid,
    pub stripe_event_id: String,
    pub event_type: String,
    pub processed: bool,
    pub payload: serde_json::Value,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Canceled,
}

impl ToString for PaymentStatus {
    fn to_string(&self) -> String {
        match self {
            PaymentStatus::Pending => "pending".to_string(),
            PaymentStatus::Processing => "processing".to_string(),
            PaymentStatus::Succeeded => "succeeded".to_string(),
            PaymentStatus::Failed => "failed".to_string(),
            PaymentStatus::Canceled => "canceled".to_string(),
        }
    }
}

impl PaymentStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(PaymentStatus::Pending),
            "processing" => Some(PaymentStatus::Processing),
            "succeeded" => Some(PaymentStatus::Succeeded),
            "failed" => Some(PaymentStatus::Failed),
            "canceled" => Some(PaymentStatus::Canceled),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatePaymentIntentRequest {
    #[schema(value_type = String, example = "123.45")]
    pub amount: BigDecimal,
    pub currency: String,
    pub order_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaymentIntentResponse {
    pub payment_intent_id: String,
    pub client_secret: String,
    #[schema(value_type = String, example = "123.45")]
    pub amount: BigDecimal,
    pub currency: String,
}
