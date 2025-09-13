use crate::errors::{AppResult};
use axum::extract::multipart::Multipart;
// use cloudinary::{
//     client::Cloudinary,
//     upload::{Upload, UploadOptions},
// };
// use std::io::Cursor;
use uuid::Uuid;

pub struct ImageService {
    // cloudinary: Cloudinary, // Temporarily disabled
}

impl ImageService {
    pub fn new(_cloud_name: &str, _api_key: &str, _api_secret: &str) -> AppResult<Self> {
        // Temporarily disabled cloudinary initialization
        Ok(Self { /* cloudinary */ })
    }

    pub async fn upload_product_image(&self, mut _multipart: Multipart) -> AppResult<String> {
        // Placeholder implementation - returns a fake URL
        let fake_image_id = Uuid::new_v4();
        Ok(format!("https://placeholder.com/products/{}.jpg", fake_image_id))
    }

    pub async fn delete_image(&self, _public_id: &str) -> AppResult<()> {
        // Placeholder implementation
        Ok(())
    }
}

// Helper function to extract public_id from Cloudinary URL
pub fn extract_public_id_from_url(url: &str) -> Option<String> {
    // Example URL: https://res.cloudinary.com/cloud_name/image/upload/v1234567890/hemp_products/abc123.jpg
    if let Some(start) = url.find("/hemp_products/") {
        let start = start + "/hemp_products/".len();
        if let Some(end) = url[start..].find('.') {
            return Some(format!("hemp_products/{}", &url[start..start + end]));
        }
    }
    None
}
