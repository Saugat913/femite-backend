use crate::repository::{OrderRepository, ProductRepository, CartRepository};
use crate::model::order::{Order, OrderStatus};
use crate::dtos::order::{CreateOrderResponse, OrderDetailsResponse, OrderItemResponse};
use crate::errors::AppError;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Clone)]
pub struct OrderService {
    repo: OrderRepository,
}

impl OrderService {
    pub fn new(repo: OrderRepository) -> Self {
        Self { repo }
    }

    pub async fn get_my_orders(&self, user_id: Uuid) -> Result<Vec<Order>, sqlx::Error> {
        self.repo.find_by_user(user_id).await
    }

    pub async fn get_all_orders(&self) -> Result<Vec<Order>, sqlx::Error> {
        self.repo.find_all().await
    }

    pub async fn update_order_status(&self, order_id: Uuid, status: String) -> Result<Order, sqlx::Error> {
        self.repo.update_status(order_id, &status).await
    }


    pub async fn create_order_from_cart(&self, user_id: Uuid) -> Result<CreateOrderResponse, AppError> {
        let cart_repo = CartRepository::new(self.repo.pool.clone());
        let product_repo = ProductRepository::new(self.repo.pool.clone());
        
        // Get user's cart
        let cart = cart_repo.get_cart_by_user(user_id).await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::Validation("No cart found for user".to_string()))?;
        
        // Get cart items
        let cart_items = cart_repo.get_cart_items(cart.id).await
            .map_err(AppError::Database)?;
        
        if cart_items.is_empty() {
            return Err(AppError::Validation("Cart is empty".to_string()));
        }
        
        // Calculate total and validate products
        let mut total = Decimal::new(0, 2);
        let mut order_items = Vec::new();
        
        for cart_item in &cart_items {
            let product = product_repo.find_by_id(cart_item.product_id).await
                .map_err(AppError::Database)?
                .ok_or_else(|| AppError::Validation(format!("Product {} not found", cart_item.product_id)))?;
            
            // Check stock availability
            if product.stock < cart_item.quantity {
                return Err(AppError::Validation(
                    format!("Insufficient stock for product {}. Available: {}, Requested: {}", 
                           product.name, product.stock, cart_item.quantity)
                ));
            }
            
            let item_price = product.price;
            let item_total = item_price * Decimal::new(cart_item.quantity as i64, 0);
            total += item_total;
            
            order_items.push((cart_item, product, item_price));
        }
        
        // Create order
        let order = self.repo.create_order(user_id, total.try_into().unwrap_or(0.0), &OrderStatus::PendingPayment.to_string()).await
            .map_err(AppError::Database)?;
        
        // Create order items
        for (cart_item, _product, item_price) in &order_items {
            self.repo.add_order_item(
                order.id, 
                cart_item.product_id, 
                cart_item.quantity, 
                (*item_price).try_into().unwrap_or(0.0)
            ).await.map_err(AppError::Database)?;
        }
        
        // Clear the cart
        cart_repo.clear_cart(cart.id).await
            .map_err(AppError::Database)?;
        
        Ok(CreateOrderResponse {
            id: order.id,
            total: order.total,
            status: order.status,
            items_count: cart_items.len() as i32,
            created_at: order.created_at,
        })
    }
    
    pub async fn get_order_details(&self, user_id: Uuid, order_id: Uuid) -> Result<OrderDetailsResponse, AppError> {
        // Get order and verify ownership
        let order = self.repo.get_by_id(order_id).await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound("Order not found".to_string()))?;
        
        if order.user_id != user_id {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }
        
        // Get order items with product details
        let items_with_products = self.repo.find_items_with_products(order_id).await
            .map_err(AppError::Database)?;
        
        let items: Vec<OrderItemResponse> = items_with_products
            .into_iter()
            .map(|(item, product_name, product_image_url)| {
                let subtotal = item.price * Decimal::new(item.quantity as i64, 0);
                OrderItemResponse {
                    id: item.id,
                    product_id: item.product_id,
                    product_name,
                    product_image_url,
                    quantity: item.quantity,
                    price: item.price,
                    subtotal,
                }
            })
            .collect();
        
        Ok(OrderDetailsResponse {
            id: order.id,
            user_id: order.user_id,
            total: order.total,
            status: order.status,
            payment_id: order.payment_id,
            items,
            created_at: order.created_at,
        })
    }
    
    pub async fn get_order_details_admin(&self, order_id: Uuid) -> Result<OrderDetailsResponse, AppError> {
        // Get order without user verification (admin access)
        let order = self.repo.get_by_id(order_id).await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound("Order not found".to_string()))?;
        
        // Get order items with product details
        let items_with_products = self.repo.find_items_with_products(order_id).await
            .map_err(AppError::Database)?;
        
        let items: Vec<OrderItemResponse> = items_with_products
            .into_iter()
            .map(|(item, product_name, product_image_url)| {
                let subtotal = item.price * Decimal::new(item.quantity as i64, 0);
                OrderItemResponse {
                    id: item.id,
                    product_id: item.product_id,
                    product_name,
                    product_image_url,
                    quantity: item.quantity,
                    price: item.price,
                    subtotal,
                }
            })
            .collect();
        
        Ok(OrderDetailsResponse {
            id: order.id,
            user_id: order.user_id,
            total: order.total,
            status: order.status,
            payment_id: order.payment_id,
            items,
            created_at: order.created_at,
        })
    }

   pub async fn pay_order(&self, user_id: Uuid, order_id: Uuid) -> Result<Option<Order>, sqlx::Error> {
        let orders = self.repo.find_by_user(user_id).await?;
        if let Some(order) = orders.into_iter().find(|o| o.id == order_id) {
            if order.status != "pending_payment" {
                return Ok(None);
            }

            // fetch order items
            let items = self.repo.find_items(order.id).await?;
            let product_repo = ProductRepository::new(self.repo.pool.clone());

            // check stock
            for item in &items {
                if let Some(product) = product_repo.find_by_id(item.product_id).await? {
                    if product.stock < item.quantity {
                        return Ok(None); // not enough stock
                    }
                } else {
                    return Ok(None); // product not found
                }
            }

            // reduce stock
            for item in &items {
                if let Some(product) = product_repo.find_by_id(item.product_id).await? {
                    let new_stock = product.stock - item.quantity;
                    product_repo.update_stock(product.id, new_stock).await?;
                }
            }

            // mark order as paid
            let updated = self.repo.update_status(order_id, &OrderStatus::Paid.to_string()).await?;
            return Ok(Some(updated));
        }
        Ok(None)
    }

}
