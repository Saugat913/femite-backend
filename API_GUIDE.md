# Hemp Product Ecommerce Backend API

This is a comprehensive Rust-based backend for a hemp products ecommerce system built with Axum, SQLx, and PostgreSQL.

## Features

- ✅ Product management with CRUD operations
- ✅ Image upload integration with Cloudinary
- ✅ Comprehensive error handling
- ✅ Database migrations with proper schema
- ✅ Admin authentication middleware
- ✅ Comprehensive test suite
- ✅ Inventory tracking and stock management
- ✅ Category management
- ✅ Cart and order functionality
- ✅ Payment integration with Stripe

## Setup

### Prerequisites

- Rust 1.70+
- PostgreSQL 13+
- Cloudinary account (for image uploads)
- Stripe account (for payments)

### Environment Configuration

1. Copy the example environment file:
```bash
cp .env.example .env
```

2. Update the `.env` file with your configuration:
```env
# Database Configuration
DATABASE_URL=postgresql://username:password@localhost:5432/hemp_backend

# Server Configuration
PORT=3000

# Security
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production

# Stripe Configuration
STRIPE_SECRET_KEY=sk_test_your_stripe_secret_key_here
STRIPE_PUBLISHABLE_KEY=pk_test_your_stripe_publishable_key_here
STRIPE_WEBHOOK_SECRET=whsec_your_webhook_secret_here

# Cloudinary Configuration
CLOUDINARY_CLOUD_NAME=your_cloud_name
CLOUDINARY_API_KEY=your_api_key
CLOUDINARY_API_SECRET=your_api_secret
```

### Database Setup

1. Create a PostgreSQL database:
```sql
CREATE DATABASE hemp_backend;
```

2. Run the application (migrations will run automatically):
```bash
cargo run
```

## API Endpoints

### Health Check
- **GET** `/health` - Health check endpoint

### Products

#### List Products
- **GET** `/api/product`
- **Response**: Array of products with pagination

```json
[
  {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "name": "Premium CBD Oil",
    "description": "High-quality CBD oil for wellness",
    "price": "29.99",
    "stock": 100,
    "image_url": "https://res.cloudinary.com/your-cloud/image/upload/hemp_products/cbd-oil.jpg",
    "low_stock_threshold": 10,
    "track_inventory": true,
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": null
  }
]
```

#### Get Product by ID
- **GET** `/api/product/{id}`
- **Response**: Single product object

#### Create Product (Admin Only)
- **POST** `/api/product`
- **Headers**: 
  - `Authorization: Bearer <admin_jwt_token>`
  - `Content-Type: application/json`
- **Body**:
```json
{
  "name": "New Hemp Product",
  "description": "Product description",
  "price": "19.99",
  "stock": 50,
  "image_url": "https://res.cloudinary.com/your-cloud/image/upload/hemp_products/product.jpg",
  "low_stock_threshold": 5,
  "track_inventory": true
}
```

#### Update Product (Admin Only)
- **PUT** `/api/product/{id}`
- **Headers**: 
  - `Authorization: Bearer <admin_jwt_token>`
  - `Content-Type: application/json`
- **Body**: (all fields optional)
```json
{
  "name": "Updated Product Name",
  "price": "24.99",
  "stock": 75
}
```

#### Delete Product (Admin Only)
- **DELETE** `/api/product/{id}`
- **Headers**: 
  - `Authorization: Bearer <admin_jwt_token>`
- **Response**: 204 No Content

### Image Upload

#### Upload Product Image (Admin Only)
- **POST** `/api/image/upload`
- **Headers**: 
  - `Authorization: Bearer <admin_jwt_token>`
  - `Content-Type: multipart/form-data`
- **Body**: Form data with `image` field containing the image file
- **Supported formats**: JPEG, PNG, WebP
- **Max file size**: 5MB
- **Response**:
```json
{
  "image_url": "https://res.cloudinary.com/your-cloud/image/upload/v1234567890/hemp_products/abc123.jpg"
}
```

## Usage Examples

### Creating a Product with Image

1. First, upload the image:
```bash
curl -X POST http://localhost:3000/api/image/upload \
  -H "Authorization: Bearer your_admin_token" \
  -F "image=@/path/to/your/image.jpg"
```

2. Use the returned image URL to create a product:
```bash
curl -X POST http://localhost:3000/api/product \
  -H "Authorization: Bearer your_admin_token" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Premium Hemp Oil",
    "description": "Organic, full-spectrum hemp oil",
    "price": "49.99",
    "stock": 100,
    "image_url": "https://res.cloudinary.com/your-cloud/image/upload/v1234567890/hemp_products/abc123.jpg",
    "low_stock_threshold": 10,
    "track_inventory": true
  }'
```

### Testing with curl

#### Get all products:
```bash
curl http://localhost:3000/api/product
```

#### Get a specific product:
```bash
curl http://localhost:3000/api/product/{product_id}
```

#### Create a product (requires admin token):
```bash
curl -X POST http://localhost:3000/api/product \
  -H "Authorization: Bearer your_admin_token" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Product",
    "description": "A test product",
    "price": "19.99",
    "stock": 50,
    "track_inventory": true
  }'
```

## Running Tests

### Unit Tests
```bash
cargo test --lib
```

### Integration Tests
```bash
# Set up a test database first
createdb hemp_backend_test
DATABASE_URL=postgresql://postgres:password@localhost:5432/hemp_backend_test cargo test --test integration_tests
```

### All Tests
```bash
cargo test
```

## Database Schema

### Products Table
```sql
CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    price NUMERIC(10,2) NOT NULL,
    stock INTEGER NOT NULL DEFAULT 0,
    image_url TEXT,
    low_stock_threshold INTEGER DEFAULT 10,
    track_inventory BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE
);
```

## Error Handling

The API uses structured error responses:

```json
{
  "error": "Short error message",
  "details": "Detailed error description"
}
```

### Common HTTP Status Codes
- `200` - Success
- `201` - Created
- `204` - No Content (for DELETE operations)
- `400` - Bad Request (validation errors, invalid file type, etc.)
- `401` - Unauthorized (missing or invalid JWT token)
- `403` - Forbidden (insufficient permissions)
- `404` - Not Found
- `413` - Payload Too Large (file size exceeds limit)
- `500` - Internal Server Error

### Image Upload Specific Errors
- File too large (max 5MB)
- Invalid file type (only JPEG, PNG, WebP allowed)
- Cloudinary upload failures
- Missing image field in multipart request

## Development

### Project Structure
```
src/
├── main.rs              # Application entry point
├── state.rs             # Application state with database pool and config
├── errors.rs            # Custom error types and handling
├── dtos/                # Data Transfer Objects
├── model/               # Database models
├── repository/          # Database access layer
├── services/            # Business logic layer
├── routes/              # HTTP route handlers
└── middleware/          # Authentication and other middleware

migrations/              # Database migration files
tests/                   # Test files
```

### Adding New Features

1. Create database migration in `migrations/`
2. Update models in `src/model/`
3. Update DTOs in `src/dtos/`
4. Implement repository methods in `src/repository/`
5. Implement business logic in `src/services/`
6. Add route handlers in `src/routes/`
7. Write tests in `tests/`

## Deployment

### Production Checklist
- [ ] Set strong JWT secret
- [ ] Configure proper CORS origins
- [ ] Set up SSL/TLS certificates
- [ ] Configure production database
- [ ] Set up proper logging levels
- [ ] Configure Cloudinary production account
- [ ] Set up Stripe production keys
- [ ] Configure proper backup strategy
- [ ] Set up monitoring and alerting

### Environment Variables for Production
Make sure to set all required environment variables in your production environment, especially:
- Use a cryptographically secure `JWT_SECRET`
- Use production Stripe keys instead of test keys
- Use production Cloudinary credentials
- Set appropriate `RUST_LOG` level for production

## License

This project is licensed under the MIT License.
