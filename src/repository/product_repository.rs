use crate::model::product::Product;
use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct ProductRepository {
    pub pool: PgPool,
}

impl ProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>, sqlx::Error> {
        sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }
    
    pub async fn create(
        &self,
        name: &str,
        description: Option<&str>,
        price: Decimal,
        stock: i32,
        image_url: Option<&str>,
        low_stock_threshold: Option<i32>,
        track_inventory: bool,
    ) -> Result<Product, sqlx::Error> {
        let id = Uuid::new_v4();
        let created_at = Utc::now();

        let rec = sqlx::query_as::<_, Product>(
            r#"
            INSERT INTO products (id, name, description, price, stock, image_url, low_stock_threshold, track_inventory, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(price)
        .bind(stock)
        .bind(image_url)
        .bind(low_stock_threshold)
        .bind(track_inventory)
        .bind(created_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(rec)
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Product>, sqlx::Error> {
        let rec = sqlx::query_as::<_, Product>(
            "SELECT * FROM products WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(rec)
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Product>, sqlx::Error> {
        let recs = sqlx::query_as::<_, Product>(
            "SELECT * FROM products ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(recs)
    }

    pub async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        price: Option<Decimal>,
        stock: Option<i32>,
        image_url: Option<&str>,
        low_stock_threshold: Option<i32>,
        track_inventory: Option<bool>,
    ) -> Result<Option<Product>, sqlx::Error> {
        // Simple approach: fetch, then update only provided fields
        if let Some(p) = self.get(id).await? {
            let new_name = name.unwrap_or(&p.name);
            let new_description = match description {
                Some(s) => Some(s.to_string()),
                None => p.description.clone(),
            };
            let new_price = price.unwrap_or(p.price);
            let new_stock = stock.unwrap_or(p.stock);
            let new_image_url = match image_url {
                Some(s) => Some(s.to_string()),
                None => p.image_url.clone(),
            };
            let new_low_stock_threshold = low_stock_threshold.or(p.low_stock_threshold);
            let new_track_inventory = track_inventory.unwrap_or(p.track_inventory);
            let updated_at = Utc::now();

            let rec = sqlx::query_as::<_, Product>(
                r#"
                UPDATE products
                SET name = $1, description = $2, price = $3, stock = $4, image_url = $5, 
                    low_stock_threshold = $6, track_inventory = $7, updated_at = $8
                WHERE id = $9
                RETURNING *
                "#,
            )
            .bind(new_name)
            .bind(new_description)
            .bind(new_price)
            .bind(new_stock)
            .bind(new_image_url)
            .bind(new_low_stock_threshold)
            .bind(new_track_inventory)
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
        let res = sqlx::query("DELETE FROM products WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn update_stock(&self, id: Uuid, stock: i32) -> Result<Product, sqlx::Error> {
        sqlx::query_as::<_, Product>("UPDATE products SET stock = $1 WHERE id = $2 RETURNING *")
            .bind(stock)
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }
}
