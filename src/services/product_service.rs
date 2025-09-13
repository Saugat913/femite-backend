use crate::repository::ProductRepository;
use crate::dtos::{NewProductDto, UpdateProductDto};
use crate::model::product::Product;
use crate::errors::{AppError, AppResult};
use uuid::Uuid;


#[derive(Clone)]
pub struct ProductService {
    repo: ProductRepository,
}

impl ProductService {
    pub fn new(repo: ProductRepository) -> Self {
        Self { repo }
    }

    pub async fn create(&self, dto: NewProductDto) -> AppResult<Product> {
        // Set defaults for optional fields
        let track_inventory = dto.track_inventory.unwrap_or(true);
        let low_stock_threshold = dto.low_stock_threshold;
        
        self.repo.create(
            &dto.name, 
            dto.description.as_deref(), 
            dto.price, 
            dto.stock,
            dto.image_url.as_deref(),
            low_stock_threshold,
            track_inventory,
        ).await.map_err(AppError::Database)
    }

    pub async fn get(&self, id: Uuid) -> AppResult<Option<Product>> {
        self.repo.get(id).await.map_err(AppError::Database)
    }

    pub async fn list(&self, limit: i64, offset: i64) -> AppResult<Vec<Product>> {
        self.repo.list(limit, offset).await.map_err(AppError::Database)
    }

    pub async fn update(&self, id: Uuid, dto: UpdateProductDto) -> AppResult<Option<Product>> {
        self.repo.update(
            id, 
            dto.name.as_deref(), 
            dto.description.as_deref(), 
            dto.price, 
            dto.stock,
            dto.image_url.as_deref(),
            dto.low_stock_threshold,
            dto.track_inventory,
        ).await.map_err(AppError::Database)
    }

    pub async fn delete(&self, id: Uuid) -> AppResult<bool> {
        self.repo.delete(id).await.map_err(AppError::Database)
    }
}
