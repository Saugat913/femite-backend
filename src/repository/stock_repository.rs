use crate::model::stock::{StockReservation, InventoryLog, InventoryChangeType, LowStockAlert};
use chrono::{Utc, Duration};
use sqlx::{PgPool, Result};
use uuid::Uuid;

#[derive(Clone)]
pub struct StockRepository {
    db: PgPool,
}

impl StockRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    // Stock Reservations
    pub async fn create_reservation(
        &self,
        product_id: Uuid,
        cart_id: Uuid,
        quantity: i32,
        expires_in_minutes: i32,
    ) -> Result<Option<StockReservation>> {
        let mut tx = self.db.begin().await?;

        // Check available stock (total stock - reserved stock)
        let available_stock: Option<Option<i64>> = sqlx::query_scalar!(
            r#"
            SELECT (p.stock - COALESCE(SUM(sr.quantity), 0)) as available_stock
            FROM products p
            LEFT JOIN stock_reservations sr ON p.id = sr.product_id AND sr.expires_at > now()
            WHERE p.id = $1 AND p.track_inventory = true
            GROUP BY p.id, p.stock
            "#,
            product_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(Some(available)) = available_stock {
            if available < quantity as i64 {
                tx.rollback().await?;
                return Ok(None); // Not enough stock
            }
        } else {
            tx.rollback().await?;
            return Ok(None); // Product not found or inventory not tracked
        }

        // Create reservation
        let reservation_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::minutes(expires_in_minutes as i64);

        let reservation = sqlx::query_as!(
            StockReservation,
            r#"
            INSERT INTO stock_reservations (id, product_id, cart_id, quantity, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, product_id, cart_id, quantity, reserved_at, expires_at, created_at
            "#,
            reservation_id,
            product_id,
            cart_id,
            quantity,
            expires_at
        )
        .fetch_one(&mut *tx)
        .await?;

        // Log the reservation
        self.log_inventory_change(
            &mut tx,
            product_id,
            InventoryChangeType::Reserved,
            -quantity, // negative because it reduces available stock
            0, // we don't change actual stock here
            0,
            Some(cart_id),
            Some(&format!("Reserved {} units for cart", quantity)),
        ).await?;

        tx.commit().await?;
        Ok(Some(reservation))
    }

    pub async fn cancel_reservation(&self, reservation_id: Uuid) -> Result<bool> {
        let mut tx = self.db.begin().await?;

        // Get reservation details before deleting
        let reservation = sqlx::query_as!(
            StockReservation,
            "SELECT id, product_id, cart_id, quantity, reserved_at, expires_at, created_at FROM stock_reservations WHERE id = $1",
            reservation_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(res) = reservation {
            // Delete reservation
            let deleted_rows = sqlx::query!(
                "DELETE FROM stock_reservations WHERE id = $1",
                reservation_id
            )
            .execute(&mut *tx)
            .await?
            .rows_affected();

            if deleted_rows > 0 {
                // Log unreservation
                self.log_inventory_change(
                    &mut tx,
                    res.product_id,
                    InventoryChangeType::Unreserved,
                    res.quantity, // positive because it increases available stock
                    0,
                    0,
                    Some(res.cart_id),
                    Some(&format!("Unreserved {} units from cart", res.quantity)),
                ).await?;

                tx.commit().await?;
                return Ok(true);
            }
        }

        tx.rollback().await?;
        Ok(false)
    }

    pub async fn cleanup_expired_reservations(&self) -> Result<i32> {
        let mut tx = self.db.begin().await?;

        // Get expired reservations for logging
        let expired_reservations = sqlx::query_as!(
            StockReservation,
            "SELECT id, product_id, cart_id, quantity, reserved_at, expires_at, created_at FROM stock_reservations WHERE expires_at <= now()"
        )
        .fetch_all(&mut *tx)
        .await?;

        // Delete expired reservations
        let deleted_rows = sqlx::query!(
            "DELETE FROM stock_reservations WHERE expires_at <= now()"
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        // Log unreservations
        for res in expired_reservations {
            self.log_inventory_change(
                &mut tx,
                res.product_id,
                InventoryChangeType::Unreserved,
                res.quantity,
                0,
                0,
                Some(res.cart_id),
                Some("Expired reservation cleanup"),
            ).await?;
        }

        tx.commit().await?;
        Ok(deleted_rows as i32)
    }

    // Stock Management
    pub async fn update_stock(
        &self,
        product_id: Uuid,
        new_stock: i32,
        change_type: InventoryChangeType,
        reference_id: Option<Uuid>,
        notes: Option<String>,
    ) -> Result<bool> {
        let mut tx = self.db.begin().await?;

        // Get current stock
        let current_stock: Option<i32> = sqlx::query_scalar!(
            "SELECT stock FROM products WHERE id = $1",
            product_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(current) = current_stock {
            let quantity_change = new_stock - current;

            // Update product stock
            let updated_rows = sqlx::query!(
                "UPDATE products SET stock = $1, updated_at = now() WHERE id = $2",
                new_stock,
                product_id
            )
            .execute(&mut *tx)
            .await?
            .rows_affected();

            if updated_rows > 0 {
                // Log the change
                self.log_inventory_change(
                    &mut tx,
                    product_id,
                    change_type,
                    quantity_change,
                    current,
                    new_stock,
                    reference_id,
                    notes.as_deref(),
                ).await?;

                tx.commit().await?;
                return Ok(true);
            }
        }

        tx.rollback().await?;
        Ok(false)
    }

    pub async fn get_available_stock(&self, product_id: Uuid) -> Result<Option<i32>> {
        let available_stock: Option<Option<i64>> = sqlx::query_scalar!(
            r#"
            SELECT (p.stock - COALESCE(SUM(sr.quantity), 0)) as available_stock
            FROM products p
            LEFT JOIN stock_reservations sr ON p.id = sr.product_id AND sr.expires_at > now()
            WHERE p.id = $1
            GROUP BY p.id, p.stock
            "#,
            product_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(available_stock.flatten().map(|s| s as i32))
    }

    pub async fn get_low_stock_alerts(&self) -> Result<Vec<LowStockAlert>> {
        let alerts = sqlx::query!(
            r#"
            SELECT 
                p.id as product_id,
                p.name as product_name,
                p.stock as current_stock,
                (p.stock - COALESCE(SUM(sr.quantity), 0)) as available_stock,
                p.low_stock_threshold as threshold
            FROM products p
            LEFT JOIN stock_reservations sr ON p.id = sr.product_id AND sr.expires_at > now()
            WHERE p.track_inventory = true 
            AND p.low_stock_threshold IS NOT NULL
            GROUP BY p.id, p.name, p.stock, p.low_stock_threshold
            HAVING (p.stock - COALESCE(SUM(sr.quantity), 0)) <= p.low_stock_threshold
            ORDER BY available_stock ASC
            "#
        )
        .fetch_all(&self.db)
        .await?;

        let low_stock_alerts = alerts
            .into_iter()
            .map(|row| LowStockAlert {
                product_id: row.product_id,
                product_name: row.product_name,
                current_stock: row.current_stock,
                available_stock: row.available_stock.unwrap_or(0) as i32,
                threshold: row.threshold.unwrap_or(0),
                is_critical: row.available_stock.unwrap_or(0) as i32 <= 0,
            })
            .collect();

        Ok(low_stock_alerts)
    }

    // Inventory Logging
    async fn log_inventory_change<'c>(
        &self,
        tx: &mut sqlx::Transaction<'c, sqlx::Postgres>,
        product_id: Uuid,
        change_type: InventoryChangeType,
        quantity_change: i32,
        previous_stock: i32,
        new_stock: i32,
        reference_id: Option<Uuid>,
        notes: Option<&str>,
    ) -> Result<()> {
        let log_id = Uuid::new_v4();
        
        sqlx::query!(
            r#"
            INSERT INTO inventory_logs (id, product_id, change_type, quantity_change, previous_stock, new_stock, reference_id, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            log_id,
            product_id,
            change_type.to_string(),
            quantity_change,
            previous_stock,
            new_stock,
            reference_id,
            notes
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn get_inventory_history(
        &self,
        product_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<InventoryLog>> {
        let logs = sqlx::query_as!(
            InventoryLog,
            "SELECT id, product_id, change_type, quantity_change, previous_stock, new_stock, reference_id, notes, created_at FROM inventory_logs WHERE product_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            product_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await?;

        Ok(logs)
    }
}
