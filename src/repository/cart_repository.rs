use crate::model::cart::{Cart, CartItem};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

#[derive(Clone)]
pub struct CartRepository {
    pub pool: PgPool,
}

impl CartRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_or_create_cart(&self, user_id: Uuid) -> Result<Cart, sqlx::Error> {
        if let Some(cart) = sqlx::query_as::<_, Cart>(
            "SELECT * FROM carts WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await? {
            return Ok(cart);
        }

        let cart = sqlx::query_as::<_, Cart>(
            "INSERT INTO carts (id, user_id, created_at) VALUES ($1, $2, $3) RETURNING *"
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(cart)
    }

    pub async fn add_item(&self, cart_id: Uuid, product_id: Uuid, quantity: i32) -> Result<CartItem, sqlx::Error> {
        sqlx::query_as::<_, CartItem>(
            "INSERT INTO cart_items (id, cart_id, product_id, quantity) VALUES ($1, $2, $3, $4) RETURNING *"
        )
        .bind(Uuid::new_v4())
        .bind(cart_id)
        .bind(product_id)
        .bind(quantity)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_cart_items(&self, cart_id: Uuid) -> Result<Vec<CartItem>, sqlx::Error> {
        sqlx::query_as::<_, CartItem>(
            "SELECT * FROM cart_items WHERE cart_id = $1 ORDER BY id"
        )
        .bind(cart_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn clear_cart(&self, cart_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM cart_items WHERE cart_id = $1", cart_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_cart_by_user(&self, user_id: Uuid) -> Result<Option<Cart>, sqlx::Error> {
        sqlx::query_as::<_, Cart>(
            "SELECT * FROM carts WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }
}
