use std::sync::Arc;

use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: Arc<String>,
    pub cloudinary_cloud_name: Arc<String>,
    pub cloudinary_api_key: Arc<String>,
    pub cloudinary_api_secret: Arc<String>,
}
