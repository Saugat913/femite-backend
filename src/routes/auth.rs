use axum::{Router, routing::post, Json, extract::State, http::StatusCode, response::IntoResponse};
use crate::services::auth_service::AuthService;
use crate::state::AppState;
use crate::repository::UserRepository;
use crate::errors::{AppResult, AppError};
use crate::middleware::validation::ValidatedJson;
use crate::dtos::{SignupDto, LoginDto, UserResponse};
use serde_json::json;

pub fn build_route() -> Router<AppState> {
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
}

#[utoipa::path(
    post,
    path = "/api/auth/signup",
    request_body = SignupDto,
    responses(
        (status = 201, description = "User created", body = UserResponse),
        (status = 400, description = "Validation error or user already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Authentication"
)]
async fn signup(State(state): State<AppState>, ValidatedJson(dto): ValidatedJson<SignupDto>) -> AppResult<impl IntoResponse> {
    let repo = UserRepository::new(state.db.clone());
    let svc = AuthService::new(repo, (*state.jwt_secret).clone());

    let user = svc.signup(dto).await?;
    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginDto,
    responses(
        (status = 200, description = "Login successful", body = inline(Object), example = json!({"token": "jwt_token_here"})),
        (status = 401, description = "Invalid credentials"),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Authentication"
)]
async fn login(State(state): State<AppState>, ValidatedJson(dto): ValidatedJson<LoginDto>) -> AppResult<impl IntoResponse> {
    let repo = UserRepository::new(state.db.clone());
    let svc = AuthService::new(repo, (*state.jwt_secret).clone());

    match svc.login(dto).await? {
        Some(token) => Ok((StatusCode::OK, Json(json!({"token": token})))),
        None => Err(AppError::Unauthorized),
    }
}
