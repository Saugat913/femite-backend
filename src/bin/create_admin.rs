use std::env;
use argon2::{password_hash::{SaltString, rand_core::OsRng}, Argon2, PasswordHasher};
use sqlx::postgres::PgPoolOptions;
use hemp_backend::repository::UserRepository;
use hemp_backend::model::user::User;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env if present
    let _ = dotenvy::dotenv();

    let email = match env::var("ADMIN_EMAIL") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("ERROR: ADMIN_EMAIL is not set. Set it in the environment.");
            eprintln!("Example: ADMIN_EMAIL=admin@example.com ADMIN_PASSWORD=secret DATABASE_URL=postgres://... cargo run --bin create_admin");
            std::process::exit(2);
        }
    };
    let password = match env::var("ADMIN_PASSWORD") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("ERROR: ADMIN_PASSWORD is not set. Set it in the environment.");
            eprintln!("Example: ADMIN_EMAIL=admin@example.com ADMIN_PASSWORD=secret DATABASE_URL=postgres://... cargo run --bin create_admin");
            std::process::exit(2);
        }
    };

    let database_url = env::var("DATABASE_URL")
        .or_else(|_| env::var("TEST_DATABASE_URL"))
        .map_err(|_| {
            eprintln!("ERROR: DATABASE_URL or TEST_DATABASE_URL must be set.");
            eprintln!("Set DATABASE_URL for production, or TEST_DATABASE_URL for local/test.");
            std::io::Error::new(std::io::ErrorKind::Other, "missing DATABASE_URL")
        })?;

    // Connect to DB
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Optionally run migrations if requested
    let run_migrations = env::var("RUN_MIGRATIONS").ok().unwrap_or_else(|| "0".into()) == "1"
        || env::var("TEST_RUN_MIGRATIONS").ok().unwrap_or_else(|| "0".into()) == "1";
    if run_migrations {
        eprintln!("Running migrations...");
        sqlx::migrate!().run(&pool).await?;
    }

    // Hash password with Argon2 (same method used in AuthService)
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Cannot hash password")
        .to_string();

    let repo = UserRepository::new(pool.clone());

    // Check if user exists
    match repo.find_by_email(&email).await? {
        Some(existing) => {
            // Update existing user to admin role and set new password hash
            let updated: User = sqlx::query_as::<_, User>(
                "UPDATE users SET role = $1, password_hash = $2 WHERE email = $3 RETURNING id, email, password_hash, role, created_at"
            )
            .bind("admin")
            .bind(&password_hash)
            .bind(&email)
            .fetch_one(&pool)
            .await?;

            println!("Updated existing user '{}' to admin (id: {}).", updated.email, updated.id);
        }
        None => {
            // Create new admin user
            let created = repo.create(&email, &password_hash, "admin").await?;
            println!("Created admin user '{}' (id: {}).", created.email, created.id);
        }
    }

    Ok(())
}

