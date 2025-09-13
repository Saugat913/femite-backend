use crate::repository::OrderRepository;
use crate::{
    middleware::auth::{AuthUser, require_admin},
    model::order::UpdateStatusDto,
    services::order_service::OrderService,
    state::AppState,
    errors::AppResult,
    dtos::order::{CreateOrderRequest, CreateOrderResponse, OrderDetailsResponse},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
};


use uuid::Uuid;

pub fn build_route() -> Router<AppState> {
    Router::new()
        .route("/", post(create_order))
        .route("/my", get(my_orders))
        .route("/all", get(all_orders))
        .route("/{id}", get(get_order_details))
        .route("/{id}/status", put(update_status))
        .route("/{id}/pay", post(pay_order))
}

#[utoipa::path(
    get,
    path = "/api/order/my",
    responses(
        (status = 200, description = "User's orders"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Orders"
)]
async fn my_orders(State(state): State<AppState>, AuthUser(claims): AuthUser) -> AppResult<impl IntoResponse> {
    let repo = OrderRepository::new(state.db.clone());
    let svc = OrderService::new(repo);

    let orders = svc.get_my_orders(claims.sub).await?;
    Ok(Json(orders))
}

#[utoipa::path(
    get,
    path = "/api/order/all",
    responses(
        (status = 200, description = "All orders (admin only)"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Orders"
)]
async fn all_orders(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> AppResult<impl IntoResponse> {
    require_admin(&claims)?;

    let repo = OrderRepository::new(state.db.clone());
    let svc = OrderService::new(repo);

    let orders = svc.get_all_orders().await?;
    Ok(Json(orders))
}

#[utoipa::path(
    put,
    path = "/api/order/{id}/status",
    params(
        ("id" = Uuid, Path, description = "Order ID")
    ),
    responses(
        (status = 200, description = "Order status updated"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "Order not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Orders"
)]
async fn update_status(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateStatusDto>,
) -> AppResult<impl IntoResponse> {
    require_admin(&claims)?;

    let repo = OrderRepository::new(state.db.clone());
    let svc = OrderService::new(repo);

    let order = svc.update_order_status(id, dto.status).await?;
    Ok(Json(order))
}
#[utoipa::path(
    post,
    path = "/api/order/{id}/pay",
    params(
        ("id" = Uuid, Path, description = "Order ID")
    ),
    responses(
        (status = 200, description = "Payment processed"),
        (status = 400, description = "Invalid order or already paid"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Orders"
)]
async fn pay_order(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(order_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let repo = OrderRepository::new(state.db.clone());
    let svc = OrderService::new(repo);

    match svc.pay_order(claims.sub, order_id).await? {
        Some(order) => Ok((StatusCode::OK, Json(order))),
        None => Err(crate::errors::AppError::Validation("Invalid order or already paid".to_string())),
    }
}

#[utoipa::path(
    post,
    path = "/api/order",
    request_body = CreateOrderRequest,
    responses(
        (status = 201, description = "Order created successfully", body = CreateOrderResponse),
        (status = 400, description = "Invalid request or empty cart"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Orders"
)]
async fn create_order(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(_dto): Json<CreateOrderRequest>,
) -> AppResult<impl IntoResponse> {
    let repo = OrderRepository::new(state.db.clone());
    let svc = OrderService::new(repo);

    match svc.create_order_from_cart(claims.sub).await {
        Ok(order) => Ok((StatusCode::CREATED, Json(order))),
        Err(e) => Err(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/order/{id}",
    params(
        ("id" = Uuid, Path, description = "Order ID")
    ),
    responses(
        (status = 200, description = "Order details retrieved", body = OrderDetailsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Order not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Orders"
)]
async fn get_order_details(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(order_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let repo = OrderRepository::new(state.db.clone());
    let svc = OrderService::new(repo);

    // Check if user is admin to allow accessing any order
    let is_admin = claims.role == "admin";
    
    let result = if is_admin {
        svc.get_order_details_admin(order_id).await
    } else {
        svc.get_order_details(claims.sub, order_id).await
    };

    match result {
        Ok(order) => Ok(Json(order)),
        Err(e) => Err(e),
    }
}
