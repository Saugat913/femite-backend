use crate::repository::CategoryRepository;
use crate::dtos::{NewCategoryDto, UpdateCategoryDto};
use crate::model::category::Category;
use uuid::Uuid;

#[derive(Clone)]
pub struct CategoryService {
    repo: CategoryRepository,
}

impl CategoryService {
    pub fn new(repo: CategoryRepository) -> Self {
        Self { repo }
    }

    pub async fn create(&self, dto: NewCategoryDto) -> Result<Category, sqlx::Error> {
        self.repo.create(&dto.name, dto.description.as_deref()).await
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<Category>, sqlx::Error> {
        self.repo.get(id).await
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Category>, sqlx::Error> {
        self.repo.list(limit, offset).await
    }

    pub async fn update(&self, id: Uuid, dto: UpdateCategoryDto) -> Result<Option<Category>, sqlx::Error> {
        self.repo.update(id, dto.name.as_deref(), dto.description.as_deref()).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        self.repo.delete(id).await
    }

    pub async fn assign_product(&self, category_id: Uuid, product_id: Uuid) -> Result<(), sqlx::Error> {
        self.repo.assign_product(category_id, product_id).await
    }
}
