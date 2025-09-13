use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi as _;
use utoipa_swagger_ui::SwaggerUi;

use crate::openapi::ApiDoc;
use crate::state::AppState;

mod dtos;
mod errors;
mod middleware;
mod model;
mod openapi;
mod repository;
mod routes;
mod services;

mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "hemp_backend=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let server_port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let server_address = format!("0.0.0.0:{}", server_port);

    let database_url =
        env::var("DATABASE_URL").expect("Expected the DATABASE_URL environment variable");
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::warn!("JWT_SECRET not set, using default (not secure for production)");
        "change_me_in_production".to_string()
    });

    // Ensure Stripe key is available if needed
    let _stripe_secret_key = env::var("STRIPE_SECRET_KEY")
        .expect("STRIPE_SECRET_KEY environment variable is required for payment processing");

    // Cloudinary configuration
    let cloudinary_cloud_name = env::var("CLOUDINARY_CLOUD_NAME")
        .expect("CLOUDINARY_CLOUD_NAME environment variable is required for image uploads");
    let cloudinary_api_key = env::var("CLOUDINARY_API_KEY")
        .expect("CLOUDINARY_API_KEY environment variable is required for image uploads");
    let cloudinary_api_secret = env::var("CLOUDINARY_API_SECRET")
        .expect("CLOUDINARY_API_SECRET environment variable is required for image uploads");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Cannot connect to database");

    // Run migrations on startup
    tracing::info!("Running database migrations...");
    sqlx::migrate!().run(&pool).await?;
    tracing::info!("Database migrations completed successfully");

    let state = AppState {
        db: pool,
        jwt_secret: std::sync::Arc::new(jwt_secret),
        cloudinary_cloud_name: std::sync::Arc::new(cloudinary_cloud_name),
        cloudinary_api_key: std::sync::Arc::new(cloudinary_api_key),
        cloudinary_api_secret: std::sync::Arc::new(cloudinary_api_secret),
    };

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any); // Configure this properly for production

    let listener = TcpListener::bind(&server_address).await?;
    tracing::info!("Server listening on {}", server_address);

    let router = routes::build_route(state)
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    axum::serve(listener, router).await?;

    Ok(())
}
