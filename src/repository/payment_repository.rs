use crate::model::payment::{Payment, PaymentWebhook};
use bigdecimal::BigDecimal;
use sqlx::{PgPool, Result};
use uuid::Uuid;

#[derive(Clone)]
pub struct PaymentRepository {
    db: PgPool,
}

impl PaymentRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        order_id: Uuid,
        stripe_payment_intent_id: String,
        amount: BigDecimal,
        currency: String,
    ) -> Result<Payment, sqlx::Error> {
        let payment_id = Uuid::new_v4();
        
        let payment = sqlx::query_as!(
            Payment,
            r#"
            INSERT INTO payments (id, order_id, stripe_payment_intent_id, amount, currency, status)
            VALUES ($1, $2, $3, $4, $5, 'pending')
            RETURNING id, order_id, stripe_payment_intent_id, amount, currency, status, payment_method, created_at, updated_at
            "#,
            payment_id,
            order_id,
            stripe_payment_intent_id,
            amount.clone(),
            currency
        )
        .fetch_one(&self.db)
        .await?;

        Ok(payment)
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Payment>> {
        let payment = sqlx::query_as!(
            Payment,
            "SELECT id, order_id, stripe_payment_intent_id, amount, currency, status, payment_method, created_at, updated_at FROM payments WHERE id = $1",
            id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(payment)
    }

    pub async fn get_by_order_id(&self, order_id: Uuid) -> Result<Option<Payment>> {
        let payment = sqlx::query_as!(
            Payment,
            "SELECT id, order_id, stripe_payment_intent_id, amount, currency, status, payment_method, created_at, updated_at FROM payments WHERE order_id = $1",
            order_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(payment)
    }

    pub async fn get_by_stripe_payment_intent_id(&self, stripe_payment_intent_id: &str) -> Result<Option<Payment>> {
        let payment = sqlx::query_as!(
            Payment,
            "SELECT id, order_id, stripe_payment_intent_id, amount, currency, status, payment_method, created_at, updated_at FROM payments WHERE stripe_payment_intent_id = $1",
            stripe_payment_intent_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(payment)
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: String,
        payment_method: Option<String>,
    ) -> Result<Option<Payment>> {
        let payment = sqlx::query_as!(
            Payment,
            r#"
            UPDATE payments 
            SET status = $1, payment_method = $2, updated_at = now() 
            WHERE id = $3
            RETURNING id, order_id, stripe_payment_intent_id, amount, currency, status, payment_method, created_at, updated_at
            "#,
            status,
            payment_method,
            id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(payment)
    }

    pub async fn update_status_by_stripe_id(
        &self,
        stripe_payment_intent_id: &str,
        status: String,
        payment_method: Option<String>,
    ) -> Result<Option<Payment>> {
        let payment = sqlx::query_as!(
            Payment,
            r#"
            UPDATE payments 
            SET status = $1, payment_method = $2, updated_at = now() 
            WHERE stripe_payment_intent_id = $3
            RETURNING id, order_id, stripe_payment_intent_id, amount, currency, status, payment_method, created_at, updated_at
            "#,
            status,
            payment_method,
            stripe_payment_intent_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(payment)
    }

    pub async fn list_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<Payment>> {
        let payments = sqlx::query_as!(
            Payment,
            "SELECT id, order_id, stripe_payment_intent_id, amount, currency, status, payment_method, created_at, updated_at FROM payments WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            status,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await?;

        Ok(payments)
    }

    // Webhook management
    pub async fn create_webhook_record(
        &self,
        stripe_event_id: String,
        event_type: String,
        payload: serde_json::Value,
    ) -> Result<PaymentWebhook> {
        let webhook_id = Uuid::new_v4();
        
        let webhook = sqlx::query_as!(
            PaymentWebhook,
            r#"
            INSERT INTO payment_webhooks (id, stripe_event_id, event_type, payload)
            VALUES ($1, $2, $3, $4)
            RETURNING id, stripe_event_id, event_type, processed, payload, error_message, created_at, processed_at
            "#,
            webhook_id,
            stripe_event_id,
            event_type,
            payload
        )
        .fetch_one(&self.db)
        .await?;

        Ok(webhook)
    }

    pub async fn mark_webhook_processed(
        &self,
        webhook_id: Uuid,
        error_message: Option<String>,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE payment_webhooks SET processed = true, processed_at = now(), error_message = $1 WHERE id = $2",
            error_message,
            webhook_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn get_webhook_by_stripe_event_id(&self, stripe_event_id: &str) -> Result<Option<PaymentWebhook>> {
        let webhook = sqlx::query_as!(
            PaymentWebhook,
            "SELECT id, stripe_event_id, event_type, processed, payload, error_message, created_at, processed_at FROM payment_webhooks WHERE stripe_event_id = $1",
            stripe_event_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(webhook)
    }
}
