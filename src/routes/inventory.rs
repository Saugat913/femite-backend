use crate::{
    middleware::auth::{AuthUser, require_admin},
    model::stock::{StockUpdateRequest, StockReservationRequest, InventoryChangeType, InventoryReport},
    repository::StockRepository,
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
struct PaginationQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

pub fn build_route() -> Router<AppState> {
    Router::new()
        .route("/products/{product_id}/stock", get(get_available_stock))
        .route("/products/{product_id}/stock", put(update_stock))
        .route("/products/{product_id}/history", get(get_inventory_history))
        .route("/reservations", post(create_reservation))
        .route("/reservations/{reservation_id}/cancel", post(cancel_reservation))
        .route("/cleanup-expired", post(cleanup_expired_reservations))
        .route("/alerts", get(get_low_stock_alerts))
        .route("/report", get(get_inventory_report))
}

#[utoipa::path(
    get,
    path = "/api/inventory/products/{product_id}/stock",
    params(("product_id" = Uuid, Path)),
    responses(
        (status = 200, description = "Available stock for product"),
        (status = 404, description = "Product not found")
    ),
    security(("bearer_auth" = [])),
    tag = "Inventory"
)]
async fn get_available_stock(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> impl IntoResponse {
    let repo = StockRepository::new(state.db.clone());

    match repo.get_available_stock(product_id).await {
        Ok(Some(available_stock)) => (
            StatusCode::OK,
            Json(json!({
                "product_id": product_id,
                "available_stock": available_stock
            })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Product not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/inventory/products/{product_id}/stock",
    params(("product_id" = Uuid, Path)),
    request_body = StockUpdateRequest,
    responses(
        (status = 200, description = "Stock updated"),
        (status = 404, description = "Product not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin required"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    tag = "Inventory"
)]
async fn update_stock(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(product_id): Path<Uuid>,
    Json(request): Json<StockUpdateRequest>,
) -> impl IntoResponse {
    // Require admin permission for stock updates
    if let Err(err) = require_admin(&claims) {
        return err.into_response();
    }

    let repo = StockRepository::new(state.db.clone());
    
    let change_type = if request.quantity > 0 {
        InventoryChangeType::StockIn
    } else {
        InventoryChangeType::StockOut
    };

    match repo.update_stock(
        product_id,
        request.quantity,
        change_type,
        None,
        request.notes,
    ).await {
        Ok(true) => (
            StatusCode::OK,
            Json(json!({
                "message": "Stock updated successfully",
                "product_id": product_id,
                "new_stock": request.quantity
            })),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Product not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/inventory/products/{product_id}/history",
    params(("product_id" = Uuid, Path)),
    responses(
        (status = 200, description = "Inventory history"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin required")
    ),
    security(("bearer_auth" = [])),
    tag = "Inventory"
)]
async fn get_inventory_history(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(product_id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse {
    // Require admin permission for inventory history
    if let Err(err) = require_admin(&claims) {
        return err.into_response();
    }

    let repo = StockRepository::new(state.db.clone());
    let limit = pagination.limit.unwrap_or(50);
    let offset = pagination.offset.unwrap_or(0);

    match repo.get_inventory_history(product_id, limit, offset).await {
        Ok(history) => (StatusCode::OK, Json(history)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/inventory/reservations",
    request_body = StockReservationRequest,
    responses(
        (status = 201, description = "Reservation created"),
        (status = 400, description = "Insufficient stock"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    tag = "Inventory"
)]
async fn create_reservation(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(request): Json<StockReservationRequest>,
) -> impl IntoResponse {
    let repo = StockRepository::new(state.db.clone());
    let expires_in_minutes = request.expires_in_minutes.unwrap_or(30);
    
    // For this example, we'll use the user ID as cart ID
    // In a real implementation, you'd get the cart ID from the request or user session
    let cart_id = Uuid::parse_str(&claims.sub.to_string()).unwrap_or_else(|_| Uuid::new_v4());

    match repo.create_reservation(
        request.product_id,
        cart_id,
        request.quantity,
        expires_in_minutes,
    ).await {
        Ok(Some(reservation)) => (StatusCode::CREATED, Json(reservation)).into_response(),
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Insufficient stock available"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/inventory/reservations/{reservation_id}/cancel",
    params(("reservation_id" = Uuid, Path)),
    responses(
        (status = 200, description = "Reservation cancelled"),
        (status = 404, description = "Reservation not found")
    ),
    security(("bearer_auth" = [])),
    tag = "Inventory"
)]
async fn cancel_reservation(
    State(state): State<AppState>,
    AuthUser(_claims): AuthUser,
    Path(reservation_id): Path<Uuid>,
) -> impl IntoResponse {
    let repo = StockRepository::new(state.db.clone());

    match repo.cancel_reservation(reservation_id).await {
        Ok(true) => (
            StatusCode::OK,
            Json(json!({"message": "Reservation cancelled successfully"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Reservation not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/inventory/cleanup-expired",
    responses((status = 200, description = "Cleanup complete")),
    security(("bearer_auth" = [])),
    tag = "Inventory"
)]
async fn cleanup_expired_reservations(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> impl IntoResponse {
    // This could be a scheduled job, but we'll expose it as an endpoint for now
    if let Err(err) = require_admin(&claims) {
        return err.into_response();
    }

    let repo = StockRepository::new(state.db.clone());

    match repo.cleanup_expired_reservations().await {
        Ok(count) => (
            StatusCode::OK,
            Json(json!({
                "message": "Expired reservations cleaned up",
                "count": count
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/inventory/alerts",
    responses((status = 200, description = "Low stock alerts", body = [crate::model::stock::LowStockAlert])),
    security(("bearer_auth" = [])),
    tag = "Inventory"
)]
async fn get_low_stock_alerts(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> impl IntoResponse {
    // Require admin permission for stock alerts
    if let Err(err) = require_admin(&claims) {
        return err.into_response();
    }

    let repo = StockRepository::new(state.db.clone());

    match repo.get_low_stock_alerts().await {
        Ok(alerts) => (StatusCode::OK, Json(alerts)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/inventory/report",
    responses((status = 200, description = "Inventory report", body = crate::model::stock::InventoryReport)),
    security(("bearer_auth" = [])),
    tag = "Inventory"
)]
async fn get_inventory_report(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> impl IntoResponse {
    // Require admin permission for inventory reports
    if let Err(err) = require_admin(&claims) {
        return err.into_response();
    }

    let repo = StockRepository::new(state.db.clone());

    // Get low stock alerts for the report
    let alerts = match repo.get_low_stock_alerts().await {
        Ok(alerts) => alerts,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
                .into_response();
        }
    };

    // Create a basic inventory report
    let low_stock_count = alerts.len() as i32;
    let out_of_stock_count = alerts.iter().filter(|a| a.is_critical).count() as i32;

    let report = InventoryReport {
        total_products: 0, // Would need additional query
        low_stock_products: low_stock_count,
        out_of_stock_products: out_of_stock_count,
        total_reserved: 0, // Would need additional query
        total_available: 0, // Would need additional query
        alerts,
    };

    (StatusCode::OK, Json(report)).into_response()
}
