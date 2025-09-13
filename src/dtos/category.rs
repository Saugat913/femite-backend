use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::model::category::Category;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewCategoryDto {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCategoryDto {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<Category> for CategoryResponse {
    fn from(c: Category) -> Self {
        CategoryResponse {
            id: c.id,
            name: c.name,
            description: c.description,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}
