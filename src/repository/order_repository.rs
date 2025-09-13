use crate::model::order::{Order, OrderItem};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Clone)]
pub struct OrderRepository {
    pub pool: PgPool,
}

impl OrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_order(&self, user_id: Uuid, total: f64, status: &str) -> Result<Order, sqlx::Error> {
        sqlx::query_as::<_, Order>(
            "INSERT INTO orders (id, user_id, total, status, created_at) VALUES ($1, $2, $3, $4, $5) RETURNING *"
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(total)
        .bind(status)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn add_order_item(&self, order_id: Uuid, product_id: Uuid, quantity: i32, price: f64) -> Result<OrderItem, sqlx::Error> {
        sqlx::query_as::<_, OrderItem>(
            "INSERT INTO order_items (id, order_id, product_id, quantity, price) VALUES ($1, $2, $3, $4, $5) RETURNING *"
        )
        .bind(Uuid::new_v4())
        .bind(order_id)
        .bind(product_id)
        .bind(quantity)
        .bind(price)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Order>, sqlx::Error> {
        sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE user_id = $1 ORDER BY created_at DESC")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn find_all(&self) -> Result<Vec<Order>, sqlx::Error> {
        sqlx::query_as::<_, Order>("SELECT * FROM orders ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn get_by_id(&self, order_id: Uuid) -> Result<Option<Order>, sqlx::Error> {
        sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE id = $1")
            .bind(order_id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn update_status(&self, order_id: Uuid, status: &str) -> Result<Order, sqlx::Error> {
        sqlx::query_as::<_, Order>(
            "UPDATE orders SET status = $1 WHERE id = $2 RETURNING *"
        )
        .bind(status)
        .bind(order_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_items(&self, order_id: Uuid) -> Result<Vec<OrderItem>, sqlx::Error> {
        sqlx::query_as::<_, OrderItem>("SELECT * FROM order_items WHERE order_id = $1")
            .bind(order_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn find_items_with_products(&self, order_id: Uuid) -> Result<Vec<(OrderItem, String, Option<String>)>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                oi.id, oi.order_id, oi.product_id, oi.quantity, oi.price,
                p.name as product_name, p.image_url as product_image_url
            FROM order_items oi
            JOIN products p ON oi.product_id = p.id
            WHERE oi.order_id = $1
            ORDER BY oi.id
            "#,
            order_id
        )
        .fetch_all(&self.pool)
        .await?;

        let result = rows
            .into_iter()
            .map(|row| {
                let order_item = OrderItem {
                    id: row.id,
                    order_id: row.order_id,
                    product_id: row.product_id,
                    quantity: row.quantity,
                    price: Decimal::from_str(&row.price.to_string()).unwrap_or_default(),
                };
                (order_item, row.product_name, row.product_image_url)
            })
            .collect();

        Ok(result)
    }
}
