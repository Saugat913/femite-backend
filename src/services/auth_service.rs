use crate::dtos::{Claims, LoginDto, SignupDto};
use crate::model::user::User;
use crate::repository::UserRepository;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{self, Argon2, PasswordVerifier as _};
use argon2::{PasswordHash, PasswordHasher};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};


#[derive(Clone)]
pub struct AuthService {
    repo: UserRepository,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(repo: UserRepository, jwt_secret: String) -> Self {
        Self { repo, jwt_secret }
    }

    pub async fn signup(&self, dto: SignupDto) -> Result<User, sqlx::Error> {
        let salt = SaltString::generate(&mut OsRng);

        let password_hash = Argon2::default()
            .hash_password(dto.password.as_bytes(), &salt)
            .expect("Cannot hash the password")
            .to_string();

        self.repo.create(&dto.email, &password_hash, "client").await
    }

    pub async fn login(&self, dto: LoginDto) -> Result<Option<String>, sqlx::Error> {
        if let Some(user) = self.repo.find_by_email(&dto.email).await? {
            let parsed_hash = PasswordHash::new(&user.password_hash)
                .expect("Cannot create password hash from raw password");
            if Argon2::default()
                .verify_password(dto.password.as_bytes(), &parsed_hash)
                .is_ok()
            {
                let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
                let claims = Claims {
                    sub: user.id,
                    email: user.email.clone(),
                    role: user.role.clone(),
                    exp,
                };
                let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
                )
                .unwrap();
                Ok(Some(token))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}
