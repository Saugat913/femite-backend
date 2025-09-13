use axum::{Router, routing::post, extract::{State}, Json, response::IntoResponse, http::StatusCode};
use crate::{middleware::auth::AuthUser, services::cart_service::CartService, state::AppState};
use crate::middleware::validation::ValidatedJson;
use crate::repository::CartRepository;
use crate::errors::AppResult;
use crate::dtos::AddToCartDto;


pub fn build_route() -> Router<AppState> {
    Router::new()
        .route("/add", post(add_to_cart))
}

#[utoipa::path(
    post,
    path = "/api/cart/add",
    request_body = AddToCartDto,
    responses(
        (status = 200, description = "Item added to cart"),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Cart"
)]
async fn add_to_cart(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    ValidatedJson(dto): ValidatedJson<AddToCartDto>,
) -> AppResult<impl IntoResponse> {
    let repo = CartRepository::new(state.db.clone());
    let svc = CartService::new(repo);

    let cart = svc.add_to_cart(claims.sub, dto).await?;
    Ok((StatusCode::OK, Json(cart)))
}
