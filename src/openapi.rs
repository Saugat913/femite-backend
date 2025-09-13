use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};


use crate::dtos::{
    NewProductDto, UpdateProductDto, ProductResponse,
    SignupDto, LoginDto, UserResponse,
    AddToCartDto, OrderResponse, CategoryResponse, NewCategoryDto, UpdateCategoryDto,
    CreateOrderRequest, CreateOrderResponse, OrderDetailsResponse, OrderItemResponse,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        // Product routes
        crate::routes::product::list_products,
        crate::routes::product::create_product,
        crate::routes::product::get_product,
        crate::routes::product::update_product,
        crate::routes::product::delete_product,
        
        // Category routes
        crate::routes::category::list_categories,
        crate::routes::category::create_category,
        crate::routes::category::get_category,
        crate::routes::category::update_category,
        crate::routes::category::delete_category,
        crate::routes::category::assign_product,
        
        // Auth routes
        crate::routes::auth::signup,
        crate::routes::auth::login,
        
        // Cart routes
        crate::routes::cart::add_to_cart,
        
        // Order routes
        crate::routes::order::create_order,
        crate::routes::order::get_order_details,
        crate::routes::order::my_orders,
        crate::routes::order::all_orders,
        crate::routes::order::update_status,
        crate::routes::order::pay_order,

        // Inventory routes
        crate::routes::inventory::get_available_stock,
        crate::routes::inventory::update_stock,
        crate::routes::inventory::get_inventory_history,
        crate::routes::inventory::create_reservation,
        crate::routes::inventory::cancel_reservation,
        crate::routes::inventory::cleanup_expired_reservations,
        crate::routes::inventory::get_low_stock_alerts,
        crate::routes::inventory::get_inventory_report,

        // Payment routes
        crate::routes::payment::create_payment_intent,
        crate::routes::payment::get_payment_by_order,
        crate::routes::payment::refund_payment,
        crate::routes::payment::handle_stripe_webhook,

        // Image upload
        crate::routes::image::upload_image,
    ),
    components(
        schemas(
            // DTOs
            NewProductDto, UpdateProductDto, ProductResponse,
            SignupDto, LoginDto, UserResponse,
            AddToCartDto, OrderResponse,
            CreateOrderRequest, CreateOrderResponse, OrderDetailsResponse, OrderItemResponse,
            CategoryResponse, NewCategoryDto, UpdateCategoryDto,

            // Models
            crate::model::product::Product,
            crate::model::product::ProductWithAvailableStock,
            crate::model::category::Category,
            crate::model::user::User,
            crate::model::cart::Cart,
            crate::model::cart::CartItem,
            crate::model::order::Order,
            crate::model::order::OrderItem,
            crate::model::order::OrderWithItems,
            crate::model::order::UpdateStatusDto,
            crate::model::payment::Payment,
            crate::model::payment::PaymentWebhook,
            crate::model::payment::CreatePaymentIntentRequest,
            crate::model::payment::PaymentIntentResponse,
            crate::model::stock::StockReservation,
            crate::model::stock::InventoryLog,
            crate::model::stock::InventoryChangeType,
            crate::model::stock::StockUpdateRequest,
            crate::model::stock::StockReservationRequest,
            crate::model::stock::LowStockAlert,
            crate::model::stock::InventoryReport,
        )
    ),
    tags(
        (name = "Products", description = "Product management endpoints"),
        (name = "Categories", description = "Category management endpoints"),
        (name = "Inventory", description = "Inventory and stock endpoints"),
        (name = "Authentication", description = "User authentication endpoints"),
        (name = "Cart", description = "Shopping cart endpoints"),
        (name = "Orders", description = "Order management endpoints"),
        (name = "Payments", description = "Payment processing endpoints"),
        (name = "Images", description = "Image upload endpoints"),
    ),
    info(
        title = "Hemp E-commerce API",
        version = "1.0.0",
        description = "A comprehensive e-commerce API built with Rust, Axum, and SQLx",
        contact(
            name = "API Support",
            email = "support@hemp-ecommerce.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development server"),
        (url = "https://api.hemp-ecommerce.com", description = "Production server")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            )
        }
    }
}

