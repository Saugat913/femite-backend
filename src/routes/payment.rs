use crate::{
    middleware::auth::AuthUser,
    model::payment::{CreatePaymentIntentRequest},
    repository::{PaymentRepository, OrderRepository},
    services::payment_service::{PaymentService, PaymentError},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use uuid::Uuid;

pub fn build_route() -> Router<AppState> {
    Router::new()
        .route("/create-payment-intent", post(create_payment_intent))
        .route("/order/{order_id}", get(get_payment_by_order))
        .route("/{payment_id}/refund", post(refund_payment))
        .route("/webhook", post(handle_stripe_webhook))
}

#[utoipa::path(
    post,
    path = "/api/payment/create-payment-intent",
    request_body = crate::model::payment::CreatePaymentIntentRequest,
    responses((status = 200, description = "Payment intent created")),
    security(("bearer_auth" = [])),
    tag = "Payments"
)]
async fn create_payment_intent(
    State(state): State<AppState>,
    AuthUser(_claims): AuthUser,
    Json(request): Json<CreatePaymentIntentRequest>,
) -> impl IntoResponse {
    let payment_repo = PaymentRepository::new(state.db.clone());
    let order_repo = OrderRepository::new(state.db.clone());
    let service = PaymentService::new(payment_repo, order_repo);

    match service.create_payment_intent(request).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(PaymentError::OrderNotFound) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Order not found"})),
        )
            .into_response(),
        Err(PaymentError::InvalidOrderStatus(status)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Invalid order status: {}", status)})),
        )
            .into_response(),
        Err(PaymentError::InvalidAmount) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid amount"})),
        )
            .into_response(),
        Err(PaymentError::StripeApiError(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Payment processing error: {}", msg)})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Internal error: {}", e)})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/payment/order/{order_id}",
    params(("order_id" = Uuid, Path)),
    responses((status = 200, description = "Payment by order")),
    security(("bearer_auth" = [])),
    tag = "Payments"
)]
async fn get_payment_by_order(
    State(state): State<AppState>,
    AuthUser(_claims): AuthUser,
    Path(order_id): Path<Uuid>,
) -> impl IntoResponse {
    let payment_repo = PaymentRepository::new(state.db.clone());
    let order_repo = OrderRepository::new(state.db.clone());
    let service = PaymentService::new(payment_repo, order_repo);

    match service.get_payment_by_order(order_id).await {
        Ok(Some(payment)) => (StatusCode::OK, Json(payment)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Payment not found for this order"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Internal error: {}", e)})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/payment/{payment_id}/refund",
    params(("payment_id" = Uuid, Path)),
    responses((status = 200, description = "Refund processed")),
    security(("bearer_auth" = [])),
    tag = "Payments"
)]
async fn refund_payment(
    State(state): State<AppState>,
    AuthUser(_claims): AuthUser,
    Path(payment_id): Path<Uuid>,
) -> impl IntoResponse {
    // TODO: Add admin check
    let payment_repo = PaymentRepository::new(state.db.clone());
    let order_repo = OrderRepository::new(state.db.clone());
    let service = PaymentService::new(payment_repo, order_repo);

    match service.refund_payment(payment_id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({"message": "Refund processed successfully"})),
        )
            .into_response(),
        Err(PaymentError::PaymentNotFound) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Payment not found"})),
        )
            .into_response(),
        Err(PaymentError::StripeApiError(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Refund processing error: {}", msg)})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Internal error: {}", e)})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/payment/webhook",
    responses((status = 200, description = "Webhook processed")),
    tag = "Payments"
)]
async fn handle_stripe_webhook(
    State(state): State<AppState>,
    body: String,
) -> impl IntoResponse {
    // Parse Stripe webhook
    let webhook_event: Result<serde_json::Value, _> = serde_json::from_str(&body);
    
    match webhook_event {
        Ok(event) => {
            let event_type = event["type"].as_str().unwrap_or("");
            let event_id = event["id"].as_str().unwrap_or("");
            
            // Store webhook for processing
            let payment_repo = PaymentRepository::new(state.db.clone());
            
            if let Err(e) = payment_repo.create_webhook_record(
                event_id.to_string(),
                event_type.to_string(),
                event.clone(),
            ).await {
                tracing::error!("Failed to store webhook: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to process webhook"})),
                )
                    .into_response();
            }

            // Process webhook based on type
            let order_repo = OrderRepository::new(state.db.clone());
            let service = PaymentService::new(payment_repo, order_repo);

            match event_type {
                "payment_intent.succeeded" => {
                    if let Some(payment_intent_id) = event["data"]["object"]["id"].as_str() {
                        if let Err(e) = service.handle_payment_succeeded(payment_intent_id).await {
                            tracing::error!("Failed to handle payment succeeded: {}", e);
                        }
                    }
                }
                "payment_intent.payment_failed" => {
                    if let Some(payment_intent_id) = event["data"]["object"]["id"].as_str() {
                        if let Err(e) = service.handle_payment_failed(payment_intent_id).await {
                            tracing::error!("Failed to handle payment failed: {}", e);
                        }
                    }
                }
                _ => {
                    tracing::info!("Unhandled webhook event type: {}", event_type);
                }
            }

            (StatusCode::OK, Json(json!({"received": true}))).into_response()
        }
        Err(e) => {
            tracing::error!("Invalid webhook payload: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid webhook payload"})),
            )
                .into_response()
        }
    }
}
