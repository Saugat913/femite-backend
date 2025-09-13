use crate::{
    errors::AppResult,
    middleware::auth::{AuthUser, require_admin},
    services::image_service::ImageService,
    state::AppState,
};
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::Serialize;


#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ImageUploadResponse {
    pub image_url: String,
}

pub fn build_route() -> Router<AppState> {
    Router::new()
        .route("/upload", post(upload_image))
}

#[utoipa::path(
    post,
    path = "/api/image/upload",
    responses((status = 201, description = "Image uploaded", body = ImageUploadResponse)),
    security(("bearer_auth" = [])),
    tag = "Images"
)]
async fn upload_image(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    multipart: Multipart,
) -> AppResult<impl IntoResponse> {
    // Require admin privileges for image uploads
    require_admin(&claims)?;

    let image_service = ImageService::new(
        &state.cloudinary_cloud_name,
        &state.cloudinary_api_key,
        &state.cloudinary_api_secret,
    )?;

    let image_url = image_service.upload_product_image(multipart).await?;

    let response = ImageUploadResponse { image_url };
    
    Ok((StatusCode::CREATED, Json(response)))
}
