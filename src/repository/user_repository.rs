use crate::model::user::User;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

#[derive(Clone)]
pub struct UserRepository {
    pub pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, email: &str, password_hash: &str, role: &str) -> Result<User, sqlx::Error> {
        let id = Uuid::new_v4();
        let created_at = Utc::now();

        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, password_hash, role, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, email, password_hash, role, created_at
            "#
        )
        .bind(id)
        .bind(email)
        .bind(password_hash)
        .bind(role)
        .bind(created_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, role, created_at FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, role, created_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }
}
