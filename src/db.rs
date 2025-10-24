use sqlx::{Pool, Postgres};
use dotenvy::dotenv;
use std::env;
use tracing::info;

pub type Db = Pool<Postgres>;

pub async fn connect() -> Db {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    info!("Connecting to database");
    let pool = Pool::<Postgres>::connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    info!("Database connection established");
    pool
}
