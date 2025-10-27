// Common test utilities and helpers
use axum::body::Body;
use axum::Router;
use serde_json::Value;
use sqlx::{PgPool, Postgres, Transaction};

use iwash::routes::create_api_router;

/// Get the test database URL from environment or use default
pub fn test_database_url() -> String {
    std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:matiecodes@localhost/iwash_test".to_string())
}

/// Create a test app with a real database connection
/// This will use a test database separate from production
pub fn create_test_app() -> Router {
    // Ensure JWT_SECRET is set for tests
    if std::env::var("JWT_SECRET").is_err() {
        unsafe {
            std::env::set_var("JWT_SECRET", "test_secret_key_for_testing");
        }
    }

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&test_database_url())
        .expect("failed to create test pool");

    Router::new()
        .nest("/api/v1", create_api_router())
        .with_state(pool)
}

/// Create a real database pool for tests that need direct DB access
pub async fn create_test_pool() -> PgPool {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&test_database_url())
        .await
        .expect("failed to connect to test database")
}

/// Parse JSON response body
pub async fn parse_json_response(body: axum::body::Body) -> Value {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// Generate a unique email for testing
pub fn unique_email(base: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}+{}@example.com", base, timestamp)
}

/// Helper to register and get a token
pub async fn register_and_get_token(email: &str, password: &str) -> String {
    use axum::http::Request;
    use serde_json::json;
    use tower::ServiceExt;

    let payload = json!({
        "email": email,
        "password": password,
        "full_name": "Test User"
    });

    let req = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let resp = create_test_app().oneshot(req).await.unwrap();
    let body = parse_json_response(resp.into_body()).await;
    body["token"].as_str().unwrap().to_string()
}

/// Clean up test data - truncate all tables
pub async fn _cleanup_test_data(pool: &PgPool) {
    sqlx::query("TRUNCATE TABLE users CASCADE")
        .execute(pool)
        .await
        .expect("failed to cleanup test data");
}

/// Setup test database - ensure schema is applied
pub async fn setup_test_database() -> PgPool {
    let pool = create_test_pool().await;
    
    // Apply schema (idempotent due to IF NOT EXISTS)
    let schema = include_str!("../../sql/test_schema.sql");
    sqlx::raw_sql(schema)
        .execute(&pool)
        .await
        .expect("failed to apply test schema");
    
    pool
}

/// Begin a database transaction for isolated testing
/// Tests can use this to ensure data is rolled back
pub async fn begin_test_transaction(pool: &PgPool) -> Transaction<'_, Postgres> {
    pool.begin().await.expect("failed to begin transaction")
}

/// Test application with HTTP client
pub struct TestApp {
    pub address: String,
    pub client: reqwest::Client,
    pub db_pool: PgPool,
}

/// Spawn test application on random port
pub async fn spawn_app() -> TestApp {
    // Set JWT_SECRET for tests
    if std::env::var("JWT_SECRET").is_err() {
        unsafe {
            std::env::set_var("JWT_SECRET", "test_secret_key_for_testing");
        }
    }

    // Create test database pool
    let db_pool = create_test_pool().await;

    // Create the Axum app
    let app = Router::new()
        .nest("/api/v1", create_api_router())
        .with_state(db_pool.clone());

    // Bind to a random port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    // Spawn the server in background
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("Failed to serve app");
    });

    // Create HTTP client
    let client = reqwest::Client::new();

    TestApp {
        address,
        client,
        db_pool,
    }
}

/// Test user helper
pub struct TestUser {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub phone: String,
    pub role: String,
}

impl TestUser {
    /// Generate a new test customer
    pub fn generate() -> Self {
        let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap();
        Self {
            email: format!("test{}@example.com", timestamp),
            password: "password123".to_string(),
            full_name: "Test User".to_string(),
            phone: format!("+123456{}", timestamp % 10000),
            role: "customer".to_string(),
        }
    }

    /// Generate a new test vendor
    pub fn generate_vendor() -> Self {
        let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap();
        Self {
            email: format!("vendor{}@example.com", timestamp),
            password: "password123".to_string(),
            full_name: "Test Vendor User".to_string(),
            phone: format!("+987654{}", timestamp % 10000),
            role: "vendor".to_string(),
        }
    }

    /// Register this user
    pub async fn register(&self, app: &TestApp) {
        use serde_json::json;

        let payload = json!({
            "full_name": self.full_name,
            "email": self.email,
            "phone": self.phone,
            "password": self.password,
            "role": self.role
        });

        let response = app
            .client
            .post(&format!("{}/api/v1/auth/register", app.address))
            .json(&payload)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), 201);
    }

    /// Login and get JWT token
    pub async fn login(&self, app: &TestApp) -> String {
        use serde_json::json;

        let payload = json!({
            "email": self.email,
            "password": self.password
        });

        let response = app
            .client
            .post(&format!("{}/api/v1/auth/login", app.address))
            .json(&payload)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status(), 200);
        let body: Value = response.json().await.unwrap();
        body["token"].as_str().unwrap().to_string()
    }
}
