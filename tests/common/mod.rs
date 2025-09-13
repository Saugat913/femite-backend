use std::env;
use std::sync::Arc;

use axum::Router;
use axum_test::TestServer;
use sqlx::{postgres::PgPoolOptions, PgPool};

use hemp_backend::{routes, state::AppState};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub async fn test_state_lazy() -> AppState {
    // Use a lazy pool so we don't connect unless a handler performs a query
    let db_url = env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/invalid".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&db_url)
        .expect("failed to create lazy pool");

    AppState {
        db: pool,
        jwt_secret: Arc::new("test_secret".to_string()),
        cloudinary_cloud_name: Arc::new("cloud_name".to_string()),
        cloudinary_api_key: Arc::new("cloud_key".to_string()),
        cloudinary_api_secret: Arc::new("cloud_secret".to_string()),
    }
}

pub async fn test_state_db() -> Option<AppState> {
    let db_url = match env::var("TEST_DATABASE_URL") {
        Ok(v) => v,
        Err(_) => return None,
    };

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .ok()?;

    // Run migrations if requested
    if env::var("TEST_RUN_MIGRATIONS").unwrap_or_else(|_| "1".to_string()) == "1" {
        if let Err(e) = sqlx::migrate!().run(&pool).await {
            eprintln!("Warning: failed to run migrations for tests: {}", e);
            return None;
        }
    }

    Some(AppState {
        db: pool,
        jwt_secret: Arc::new("test_secret".to_string()),
        cloudinary_cloud_name: Arc::new("cloud_name".to_string()),
        cloudinary_api_key: Arc::new("cloud_key".to_string()),
        cloudinary_api_secret: Arc::new("cloud_secret".to_string()),
    })
}

pub async fn app_with_state(state: AppState) -> Router {
    routes::build_route(state)
}

pub fn jwt_for(role: &str) -> String {
    #[derive(Debug, Serialize, Deserialize)]
    struct Claims { sub: Uuid, email: String, role: String, exp: usize }
    let claims = Claims {
        sub: Uuid::new_v4(),
        email: format!("{}@example.com", role),
        role: role.to_string(),
        exp: 4102444800,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(b"test_secret")).unwrap()
}

pub fn jwt_user() -> String { jwt_for("user") }
pub fn jwt_admin() -> String { jwt_for("admin") }

pub async fn test_server_lazy() -> TestServer {
    let state = test_state_lazy().await;
    let app = app_with_state(state).await;
    TestServer::new(app).expect("failed to start test server")
}

