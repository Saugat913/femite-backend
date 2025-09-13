use crate::repository::CartRepository;
use crate::dtos::AddToCartDto;
use crate::model::cart::Cart;

#[derive(Clone)]
pub struct CartService {
    repo: CartRepository,
}

impl CartService {
    pub fn new(repo: CartRepository) -> Self {
        Self { repo }
    }

    pub async fn add_to_cart(&self, user_id: uuid::Uuid, dto: AddToCartDto) -> Result<Cart, sqlx::Error> {
        let cart = self.repo.get_or_create_cart(user_id).await?;
        self.repo.add_item(cart.id, dto.product_id, dto.quantity).await?;
        Ok(cart)
    }
}
