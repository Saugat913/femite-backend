use crate::dtos::Claims;
use crate::errors::AppError;
use crate::state::AppState;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use jsonwebtoken::{DecodingKey, Validation, decode};


pub struct AuthUser(pub Claims);

impl FromRequestParts<AppState> for AuthUser
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .ok_or((StatusCode::UNAUTHORIZED, "missing auth".to_string()))?;
        let token = auth_header
            .to_str()
            .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid header".to_string()))?;
        let token = token
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "invalid token".to_string()))?;

        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid token".to_string()))?;

        Ok(AuthUser(decoded.claims))
    }
}

pub fn require_admin(claims: &Claims) -> Result<(), AppError> {
    if claims.role == "admin" {
        Ok(())
    } else {
        Err(AppError::Unauthorized)
    }
}
