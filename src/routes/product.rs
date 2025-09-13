use crate::{
    dtos::{NewProductDto, ProductResponse, UpdateProductDto},
    errors::{AppError, AppResult},
    middleware::auth::{AuthUser, require_admin},
    middleware::validation::ValidatedJson,
    repository::ProductRepository,
    services::product_service::ProductService,
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get},
};
use uuid::Uuid;
pub fn build_route() -> Router<AppState> {
    Router::new()
        .route("/", get(list_products).post(create_product))
        .route(
            "/{id}",
            get(get_product).put(update_product).delete(delete_product),
        )
}

#[utoipa::path(
    get,
    path = "/api/product",
    responses(
        (status = 200, description = "List of products", body = [ProductResponse]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Products"
)]
async fn list_products(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    let repo = ProductRepository::new(state.db.clone());
    let svc = ProductService::new(repo);

    let products = svc.list(50, 0).await?;
    let res: Vec<ProductResponse> = products.into_iter().map(|p| p.into()).collect();
    
    Ok((StatusCode::OK, Json(res)))
}

#[utoipa::path(
    post,
    path = "/api/product",
    request_body = NewProductDto,
    responses(
        (status = 201, description = "Product created", body = ProductResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Products"
)]
async fn create_product(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    ValidatedJson(payload): ValidatedJson<NewProductDto>,
) -> AppResult<impl IntoResponse> {
    require_admin(&claims)?;

    let repo = ProductRepository::new(state.db.clone());
    let svc = ProductService::new(repo);

    let product = svc.create(payload).await?;
    
    Ok((StatusCode::CREATED, Json(ProductResponse::from(product))))
}

#[utoipa::path(
    get,
    path = "/api/product/{id}",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Product found", body = ProductResponse),
        (status = 404, description = "Product not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Products"
)]
async fn get_product(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<impl IntoResponse> {
    let repo = ProductRepository::new(state.db.clone());
    let svc = ProductService::new(repo);

    match svc.get(id).await? {
        Some(product) => Ok((StatusCode::OK, Json(ProductResponse::from(product)))),
        None => Err(AppError::NotFound(format!("Product with id {} not found", id))),
    }
}

#[utoipa::path(
    put,
    path = "/api/product/{id}",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    request_body = UpdateProductDto,
    responses(
        (status = 200, description = "Product updated", body = ProductResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "Product not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Products"
)]
async fn update_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    AuthUser(claims): AuthUser,
    ValidatedJson(payload): ValidatedJson<UpdateProductDto>,
) -> AppResult<impl IntoResponse> {
    require_admin(&claims)?;

    let repo = ProductRepository::new(state.db.clone());
    let svc = ProductService::new(repo);

    match svc.update(id, payload).await? {
        Some(product) => Ok((StatusCode::OK, Json(ProductResponse::from(product)))),
        None => Err(AppError::NotFound(format!("Product with id {} not found", id))),
    }
}

#[utoipa::path(
    delete,
    path = "/api/product/{id}",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    responses(
        (status = 204, description = "Product deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "Product not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Products"
)]
async fn delete_product(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    require_admin(&claims)?;

    let repo = ProductRepository::new(state.db.clone());
    let svc = ProductService::new(repo);

    match svc.delete(id).await? {
        true => Ok(StatusCode::NO_CONTENT),
        false => Err(AppError::NotFound(format!("Product with id {} not found", id))),
    }
}
