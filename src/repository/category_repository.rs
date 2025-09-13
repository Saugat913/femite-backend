use crate::model::category::Category;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct CategoryRepository {
    pub pool: PgPool,
}

impl CategoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, name: &str, description: Option<&str>) -> Result<Category, sqlx::Error> {
        let id = Uuid::new_v4();
        let created_at = Utc::now();

        sqlx::query_as::<_, Category>(
            r#"
            INSERT INTO categories (id, name, description, created_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, description, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(created_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Category>, sqlx::Error> {
        sqlx::query_as::<_, Category>(
            "SELECT id, name, description, created_at, updated_at FROM categories WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Category>, sqlx::Error> {
        sqlx::query_as::<_, Category>(
            "SELECT id, name, description, created_at, updated_at FROM categories ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update(&self, id: Uuid, name: Option<&str>, description: Option<&str>) -> Result<Option<Category>, sqlx::Error> {
        if let Some(c) = self.get(id).await? {
            let new_name = name.unwrap_or(&c.name);
            let new_description = description.or(c.description.as_deref());
            let updated_at = Utc::now();

            let rec = sqlx::query_as::<_, Category>(
                r#"
                UPDATE categories
                SET name = $1, description = $2, updated_at = $3
                WHERE id = $4
                RETURNING id, name, description, created_at, updated_at
                "#
            )
            .bind(new_name)
            .bind(new_description)
            .bind(updated_at)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

            Ok(Some(rec))
        } else {
            Ok(None)
        }
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let res = sqlx::query("DELETE FROM categories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn assign_product(&self, category_id: Uuid, product_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO product_categories (product_id, category_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(product_id)
        .bind(category_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
