pub mod auth;
pub mod cart;
pub mod category;
pub mod image;
pub mod inventory;
pub mod order;
pub mod payment;
pub mod product;

use crate::{ state::AppState, openapi::ApiDoc};
use axum::{routing::get, Router, Json};
use utoipa::OpenApi;


pub fn build_route(state: AppState) -> Router {
  
    let router = Router::new()
        .nest("/product", product::build_route())
        .nest("/category", category::build_route())
        .nest("/auth", auth::build_route())
        .nest("/cart", cart::build_route())
        .nest("/image", image::build_route())
        .nest("/order", order::build_route())
        .nest("/payment", payment::build_route())
        .nest("/inventory", inventory::build_route());

    let api_router = Router::new()
        .nest("/api", router)
        .route("/health", get(|| async { "OK" }))
        .with_state(state);
    return api_router;
}
