# Hemp Backend - Rust E-commerce API

A comprehensive e-commerce backend API built with Rust, Axum, SQLx, and PostgreSQL, featuring Stripe payment integration and advanced inventory management.

## Features

### Core Features
- 🔐 **Authentication & Authorization** - JWT-based auth with role-based access control
- 🛍️ **Product Management** - CRUD operations with categories and inventory tracking
- 🛒 **Shopping Cart** - Persistent shopping cart with item management
- 📦 **Order Management** - Complete order lifecycle with status tracking
- 💳 **Payment Processing** - Stripe integration with webhooks
- 📊 **Inventory Management** - Stock tracking, reservations, and low-stock alerts

### Advanced Features
- 🔄 **Stock Reservations** - Automatic stock reservation for cart items
- 📈 **Inventory Tracking** - Detailed inventory logs and history
- ⚠️ **Low Stock Alerts** - Configurable stock thresholds and alerts
- 🎯 **Database Migrations** - Automated schema management with SQLx
- 🚀 **Performance** - Optimized queries with proper indexing
- 🛡️ **Security** - CORS, input validation, and secure payment handling

## Tech Stack

- **Language**: Rust
- **Web Framework**: Axum
- **Database**: PostgreSQL with SQLx
- **Payment**: Stripe
- **Authentication**: JWT
- **Logging**: Tracing
- **Validation**: Validator crate

## Quick Start

### Prerequisites

- Rust 1.70+ 
- PostgreSQL 12+
- Stripe account (for payments)

### Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd hemp_backend
   ```

2. **Set up environment variables**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Set up PostgreSQL**
   ```bash
   # Create database
   createdb hemp_backend
   
   # Update DATABASE_URL in .env
   DATABASE_URL=postgresql://username:password@localhost:5432/hemp_backend
   ```

4. **Install SQLx CLI (for migrations)**
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   ```

5. **Run migrations**
   ```bash
   sqlx migrate run
   ```

6. **Build and run**
   ```bash
   cargo run
   ```

The server will start on `http://localhost:3000` (or the port specified in `.env`).

## API Documentation

The API is fully documented with OpenAPI 3.0 specification. Once the server is running, you can access:

- **Interactive Swagger UI**: `http://localhost:3000/docs`
- **OpenAPI JSON Spec**: `http://localhost:3000/api-docs/openapi.json`

The documentation includes all endpoints with detailed request/response schemas, authentication requirements, and example payloads.

## API Endpoints

### Authentication
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - User login
- `GET /api/auth/me` - Get current user

### Products
- `GET /api/product` - List products
- `POST /api/product` - Create product (admin)
- `GET /api/product/{id}` - Get product by ID
- `PUT /api/product/{id}` - Update product (admin)
- `DELETE /api/product/{id}` - Delete product (admin)

### Categories
- `GET /api/category` - List categories
- `POST /api/category` - Create category (admin)
- `GET /api/category/{id}` - Get category by ID

### Shopping Cart
- `POST /api/cart/add` - Add item to cart

### Orders
- `POST /api/order` - Create order from cart
- `GET /api/order/my` - List user's orders
- `GET /api/order/all` - List all orders (admin only)
- `GET /api/order/{id}` - Get order details with items
- `PUT /api/order/{id}/status` - Update order status (admin)
- `POST /api/order/{id}/pay` - Process order payment

### Payments
- `POST /api/payment/create-payment-intent` - Create Stripe payment intent
- `GET /api/payment/order/{order_id}` - Get payment for order
- `POST /api/payment/{payment_id}/refund` - Process refund (admin)
- `POST /api/payment/webhook` - Stripe webhook endpoint

### Inventory Management
- `GET /api/inventory/products/{product_id}/stock` - Get available stock
- `PUT /api/inventory/products/{product_id}/stock` - Update stock (admin)
- `GET /api/inventory/products/{product_id}/history` - Get inventory history (admin)
- `POST /api/inventory/reservations` - Create stock reservation
- `POST /api/inventory/reservations/{id}/cancel` - Cancel reservation
- `GET /api/inventory/alerts` - Get low stock alerts (admin)
- `GET /api/inventory/report` - Get inventory report (admin)

### Utility
- `GET /health` - Health check

## Database Schema

The application uses the following main tables:
- `users` - User accounts with roles
- `products` - Product catalog with inventory tracking
- `categories` - Product categories
- `product_categories` - Many-to-many product-category relationships
- `carts` & `cart_items` - Shopping cart functionality
- `orders` & `order_items` - Order management
- `payments` - Payment tracking with Stripe integration
- `payment_webhooks` - Stripe webhook logs
- `stock_reservations` - Inventory reservations
- `inventory_logs` - Inventory change history

## Configuration

### Environment Variables

| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Yes | - |
| `PORT` | Server port | No | 3000 |
| `JWT_SECRET` | JWT signing secret | Yes | - |
| `STRIPE_SECRET_KEY` | Stripe secret key | Yes | - |
| `STRIPE_WEBHOOK_SECRET` | Stripe webhook secret | No | - |
| `RUST_LOG` | Logging configuration | No | info |

### Stripe Setup

1. Create a Stripe account at [stripe.com](https://stripe.com)
2. Get your API keys from the Stripe dashboard
3. Set up a webhook endpoint pointing to `/api/payment/webhook`
4. Configure the webhook to send `payment_intent.succeeded` and `payment_intent.payment_failed` events

## Development

### Running Tests
```bash
# Run unit tests
cargo test

# Test order functionality (requires running server)
./test_order_functionality.sh

# Test OpenAPI documentation (requires running server)
./test_openapi.sh
```

### Database Migrations
```bash
# Create a new migration
sqlx migrate add <migration_name>

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Code Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

## Production Deployment

### Security Checklist
- [ ] Set strong `JWT_SECRET`
- [ ] Use production Stripe keys
- [ ] Configure CORS properly
- [ ] Set up SSL/TLS
- [ ] Use environment-specific database
- [ ] Set up monitoring and logging
- [ ] Configure rate limiting

### Docker Deployment
```dockerfile
# Dockerfile example
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/hemp_backend .
EXPOSE 3000
CMD ["./hemp_backend"]
```

## API Usage Examples

### Order Management Workflow

#### 1. Add Items to Cart
```bash
curl -X POST http://localhost:3000/api/cart/add \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "product_id": "123e4567-e89b-12d3-a456-426614174000",
    "quantity": 2
  }'
```

#### 2. Create Order from Cart
```bash
curl -X POST http://localhost:3000/api/order \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "notes": "Special delivery instructions"
  }'
```

#### 3. Get Order Details
```bash
curl http://localhost:3000/api/order/{order_id} \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### 4. Process Order Payment
```bash
curl -X POST http://localhost:3000/api/order/{order_id}/pay \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### 5. Update Order Status (Admin)
```bash
curl -X PUT http://localhost:3000/api/order/{order_id}/status \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ADMIN_JWT_TOKEN" \
  -d '{
    "status": "shipped"
  }'
```

### Creating a Payment Intent
```bash
curl -X POST http://localhost:3000/api/payment/create-payment-intent \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "amount": 29.99,
    "currency": "usd",
    "order_id": "123e4567-e89b-12d3-a456-426614174000"
  }'
```

### Managing Inventory
```bash
# Update stock
curl -X PUT http://localhost:3000/api/inventory/products/{product_id}/stock \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ADMIN_JWT_TOKEN" \
  -d '{
    "quantity": 100,
    "notes": "Initial stock"
  }'

# Get low stock alerts
curl http://localhost:3000/api/inventory/alerts \
  -H "Authorization: Bearer ADMIN_JWT_TOKEN"
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support, email support@yourcompany.com or create an issue in the repository.
