use crate::model::payment::{Payment, PaymentIntentResponse, CreatePaymentIntentRequest, PaymentStatus};
use crate::model::order::OrderStatus;
use crate::repository::{PaymentRepository, OrderRepository};
use bigdecimal::{BigDecimal, ToPrimitive};
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

pub struct PaymentService {
    payment_repo: PaymentRepository,
    order_repo: OrderRepository,
    stripe_secret_key: String,
}

impl PaymentService {
    pub fn new(payment_repo: PaymentRepository, order_repo: OrderRepository) -> Self {
        let stripe_secret_key = env::var("STRIPE_SECRET_KEY")
            .expect("STRIPE_SECRET_KEY environment variable is required");
        
        Self {
            payment_repo,
            order_repo,
            stripe_secret_key,
        }
    }

    pub async fn create_payment_intent(
        &self,
        request: CreatePaymentIntentRequest,
    ) -> Result<PaymentIntentResponse, PaymentError> {
        // Validate order exists and is ready for payment
        let order = self.order_repo.get_by_id(request.order_id).await
            .map_err(|e| PaymentError::Database(e.to_string()))?;
        
        let order = order.ok_or(PaymentError::OrderNotFound)?;
        
        // Check if order is in correct status for payment
        if order.status != OrderStatus::PendingPayment.to_string() {
            return Err(PaymentError::InvalidOrderStatus(order.status));
        }

        // Convert amount to cents for Stripe (Stripe expects integer cents)
        let amount_cents = (&request.amount * BigDecimal::from(100i32))
            .to_i64()
            .ok_or(PaymentError::InvalidAmount)?;

        // Create Stripe PaymentIntent
        let client = reqwest::Client::new();
        let mut params = HashMap::new();
        params.insert("amount", amount_cents.to_string());
        params.insert("currency", request.currency.clone());
        params.insert("automatic_payment_methods[enabled]", "true".to_string());
        params.insert("metadata[order_id]", request.order_id.to_string());

        let response = client
            .post("https://api.stripe.com/v1/payment_intents")
            .header("Authorization", format!("Bearer {}", self.stripe_secret_key))
            .form(&params)
            .send()
            .await
            .map_err(|e| PaymentError::StripeApiError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown Stripe API error".to_string());
            return Err(PaymentError::StripeApiError(error_text));
        }

        let stripe_response: serde_json::Value = response.json().await
            .map_err(|e| PaymentError::StripeApiError(e.to_string()))?;

        let payment_intent_id = stripe_response["id"]
            .as_str()
            .ok_or(PaymentError::StripeApiError("Missing payment intent ID".to_string()))?;

        let client_secret = stripe_response["client_secret"]
            .as_str()
            .ok_or(PaymentError::StripeApiError("Missing client secret".to_string()))?;

        // Create payment record in database
        let _payment = self.payment_repo.create(
            request.order_id,
            payment_intent_id.to_string(),
            request.amount.clone(),
            request.currency.clone(),
        ).await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

        // Update order status to payment_processing
        self.order_repo.update_status(
            request.order_id,
            &OrderStatus::PaymentProcessing.to_string(),
        ).await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

        Ok(PaymentIntentResponse {
            payment_intent_id: payment_intent_id.to_string(),
            client_secret: client_secret.to_string(),
            amount: request.amount,
            currency: request.currency,
        })
    }

    pub async fn handle_payment_succeeded(&self, payment_intent_id: &str) -> Result<(), PaymentError> {
        // Update payment status
        let payment = self.payment_repo.update_status_by_stripe_id(
            payment_intent_id,
            PaymentStatus::Succeeded.to_string(),
            None,
        ).await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

        if let Some(payment) = payment {
            // Update order status to paid
            self.order_repo.update_status(
                payment.order_id,
                &OrderStatus::Paid.to_string(),
            ).await
            .map_err(|e| PaymentError::Database(e.to_string()))?;

            // Here you would typically:
            // 1. Convert stock reservations to actual sales
            // 2. Update inventory
            // 3. Send confirmation emails
            // 4. Trigger order fulfillment process
        }

        Ok(())
    }

    pub async fn handle_payment_failed(&self, payment_intent_id: &str) -> Result<(), PaymentError> {
        // Update payment status
        let payment = self.payment_repo.update_status_by_stripe_id(
            payment_intent_id,
            PaymentStatus::Failed.to_string(),
            None,
        ).await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

        if let Some(payment) = payment {
            // Update order status back to pending_payment
            self.order_repo.update_status(
                payment.order_id,
                &OrderStatus::PendingPayment.to_string(),
            ).await
            .map_err(|e| PaymentError::Database(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn get_payment_by_order(&self, order_id: Uuid) -> Result<Option<Payment>, PaymentError> {
        self.payment_repo.get_by_order_id(order_id).await
            .map_err(|e| PaymentError::Database(e.to_string()))
    }

    pub async fn refund_payment(&self, payment_id: Uuid) -> Result<(), PaymentError> {
        let payment = self.payment_repo.get_by_id(payment_id).await
            .map_err(|e| PaymentError::Database(e.to_string()))?;

        let payment = payment.ok_or(PaymentError::PaymentNotFound)?;

        // Create refund with Stripe
        let client = reqwest::Client::new();
        let mut params = HashMap::new();
        params.insert("payment_intent", payment.stripe_payment_intent_id.clone());

        let response = client
            .post("https://api.stripe.com/v1/refunds")
            .header("Authorization", format!("Bearer {}", self.stripe_secret_key))
            .form(&params)
            .send()
            .await
            .map_err(|e| PaymentError::StripeApiError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown Stripe API error".to_string());
            return Err(PaymentError::StripeApiError(error_text));
        }

        // Update payment status
        self.payment_repo.update_status(
            payment_id,
            PaymentStatus::Canceled.to_string(), // Using canceled for refunded
            None,
        ).await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

        // Update order status
        self.order_repo.update_status(
            payment.order_id,
            &OrderStatus::Refunded.to_string(),
        ).await
        .map_err(|e| PaymentError::Database(e.to_string()))?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Stripe API error: {0}")]
    StripeApiError(String),
    
    #[error("Order not found")]
    OrderNotFound,
    
    #[error("Payment not found")]
    PaymentNotFound,
    
    #[error("Invalid order status: {0}")]
    InvalidOrderStatus(String),
    
    #[error("Invalid amount")]
    InvalidAmount,
}
