use crate::dtos::{CategoryResponse, NewCategoryDto, UpdateCategoryDto};
use crate::repository::CategoryRepository;
use crate::{services::category_service::CategoryService, state::AppState};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};

use serde_json::json;
use uuid::Uuid;

pub fn build_route() -> Router<AppState> {
    Router::new()
        .route("/", get(list_categories).post(create_category))
        .route(
            "/{id}",
            get(get_category)
                .put(update_category)
                .delete(delete_category),
        )
        .route("/{id}/assign/{product_id}", post(assign_product))
}

#[utoipa::path(
    get,
    path = "/api/category",
    responses(
        (status = 200, description = "List categories", body = [CategoryResponse]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Categories"
)]
async fn list_categories(State(state): State<AppState>) -> impl IntoResponse {
    let repo = CategoryRepository::new(state.db.clone());
    let svc = CategoryService::new(repo);

    match svc.list(50, 0).await {
        Ok(cats) => {
            let res: Vec<CategoryResponse> = cats.into_iter().map(|c| c.into()).collect();
            (StatusCode::OK, Json(res)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/category",
    request_body = NewCategoryDto,
    responses(
        (status = 201, description = "Category created", body = CategoryResponse),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Categories"
)]
async fn create_category(
    State(state): State<AppState>,
    // AuthUser(claims): AuthUser,
    Json(payload): Json<NewCategoryDto>,
) -> impl IntoResponse {
    // if let Err(err) = require_admin(&claims) {
    //     return err.into_response();
    // }

    let repo = CategoryRepository::new(state.db.clone());
    let svc = CategoryService::new(repo);

    match svc.create(payload).await {
        Ok(c) => (StatusCode::CREATED, Json(CategoryResponse::from(c))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/category/{id}",
    params(("id" = Uuid, Path, description = "Category ID")),
    responses(
        (status = 200, description = "Category found", body = CategoryResponse),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Categories"
)]
async fn get_category(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let repo = CategoryRepository::new(state.db.clone());
    let svc = CategoryService::new(repo);

    match svc.get(id).await {
        Ok(Some(c)) => (StatusCode::OK, Json(CategoryResponse::from(c))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/category/{id}",
    params(("id" = Uuid, Path, description = "Category ID")),
    request_body = UpdateCategoryDto,
    responses(
        (status = 200, description = "Category updated", body = CategoryResponse),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Categories"
)]
async fn update_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    // AuthUser(claims): AuthUser,
    Json(payload): Json<UpdateCategoryDto>,
) -> impl IntoResponse {
    // if let Err(err) = require_admin(&claims) {
    //     return err.into_response();
    // }

    let repo = CategoryRepository::new(state.db.clone());
    let svc = CategoryService::new(repo);

    match svc.update(id, payload).await {
        Ok(Some(c)) => (StatusCode::OK, Json(CategoryResponse::from(c))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/category/{id}",
    params(("id" = Uuid, Path, description = "Category ID")),
    responses(
        (status = 204, description = "Category deleted"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Categories"
)]
async fn delete_category(
    State(state): State<AppState>,
    // AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // if let Err(err) = require_admin(&claims) {
    //     return err.into_response();
    // }

    let repo = CategoryRepository::new(state.db.clone());
    let svc = CategoryService::new(repo);

    match svc.delete(id).await {
        Ok(true) => (StatusCode::NO_CONTENT).into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/category/{id}/assign/{product_id}",
    params(("id" = Uuid, Path), ("product_id" = Uuid, Path)),
    responses(
        (status = 204, description = "Product assigned to category"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Categories"
)]
async fn assign_product(
    State(state): State<AppState>,
    // AuthUser(claims): AuthUser,
    Path((id, product_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    // if let Err(err) = require_admin(&claims) {
    //     return err.into_response();
    // }

    let repo = CategoryRepository::new(state.db.clone());
    let svc = CategoryService::new(repo);

    match svc.assign_product(id, product_id).await {
        Ok(_) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
