# iWash Backend - API Documentation

## 🏗️ Project Structure

The iWash backend follows a clean, modular architecture designed for scalability:

```
src/
├── main.rs              # Binary entry point, server initialization
├── lib.rs               # Library crate, module exports
│
├── config/              # Configuration management
│   └── mod.rs          # Future: env vars, app settings
│
├── db/                  # Database layer
│   └── mod.rs          # SQLx connection pool setup
│
├── models/              # Domain models & DTOs
│   ├── mod.rs          # Module aggregator
│   └── user.rs         # User model, request/response types
│
├── handlers/            # Business logic / request handlers
│   ├── mod.rs          # Module aggregator
│   ├── auth.rs         # Register, login handlers
│   ├── users.rs        # CRUD handlers for users
│   └── health.rs       # Health check handler
│
├── routes/              # Route definitions (thin layer)
│   ├── mod.rs          # API router aggregator
│   ├── auth.rs         # Auth routes
│   ├── users.rs        # User CRUD routes
│   └── health.rs       # Health routes
│
├── middleware/          # Request middleware
│   ├── mod.rs          # Module aggregator
│   └── auth.rs         # JWT authentication extractor
│
├── errors/              # Error handling
│   └── mod.rs          # AppError, AppResult, error responses
│
├── services/            # Business services (future)
│   └── mod.rs          # Placeholder for service layer
│
└── utils/               # Utilities (future)
    └── mod.rs          # Placeholder for helpers
```

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+
- PostgreSQL 14+
- `.env` file configured

### Setup

1. **Clone and configure**
   ```bash
   cp .env.example .env
   # Edit .env with your DATABASE_URL and JWT_SECRET
   ```

2. **Apply database schema**
   ```bash
   psql "$DATABASE_URL" -f sql/schema.sql
   ```

3. **Build and run**
   ```bash
   cargo build
   RUST_LOG=info cargo run
   ```

Server starts at `http://127.0.0.1:8080`

## 📍 API Endpoints

All API endpoints are prefixed with `/api/v1`:

### Health Check
- `GET /api/v1/health` - Check API health (public)

### Authentication
- `POST /api/v1/auth/register` - Register a new user
- `POST /api/v1/auth/login` - Login and get JWT token

### Users (requires JWT)
- `GET /api/v1/users/me` - Get current user profile
- `GET /api/v1/users` - List all users
- `POST /api/v1/users` - Create a new user
- `GET /api/v1/users/{id}` - Get user by ID
- `PATCH /api/v1/users/{id}` - Update user
- `DELETE /api/v1/users/{id}` - Delete user

## 📝 Example Requests

### Register
```bash
curl -X POST http://127.0.0.1:8080/api/v1/auth/register \
  -H 'Content-Type: application/json' \
  -d '{
    "email": "alice@example.com",
    "password": "SecureP@ss123",
    "full_name": "Alice Liddell",
    "phone": "+15551234567"
  }'
```

### Login
```bash
curl -X POST http://127.0.0.1:8080/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{
    "email": "alice@example.com",
    "password": "SecureP@ss123"
  }'
```

### Get Current User Profile
```bash
curl http://127.0.0.1:8080/api/v1/users/me \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### List Users
```bash
curl http://127.0.0.1:8080/api/v1/users \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### Update User
```bash
curl -X PATCH http://127.0.0.1:8080/api/v1/users/{id} \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "full_name": "Alice Smith"
  }'
```

## 🧪 Testing

Run tests:
```bash
cargo test
```

Run specific test:
```bash
cargo test health_check
```

## 🔧 Development

### Adding a New Feature

1. **Create Model** (`src/models/your_model.rs`)
   - Define domain entities and DTOs

2. **Create Handler** (`src/handlers/your_feature.rs`)
   - Implement business logic

3. **Create Routes** (`src/routes/your_feature.rs`)
   - Define HTTP routes

4. **Register Module**
   - Add to `src/handlers/mod.rs`
   - Add to `src/routes/mod.rs`
   - Nest in `src/routes/mod.rs::create_api_router()`

### Code Style
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Follow Rust naming conventions

## 🔐 Security

- Passwords are hashed using bcrypt
- JWT tokens expire after 24 hours
- All user endpoints require authentication
- Sensitive data (password hashes) never exposed in API responses

## 🌐 Environment Variables

### Production (.env)
- `DATABASE_URL` - PostgreSQL connection string for production
- `JWT_SECRET` - Secret key for JWT signing
- `RUST_LOG` - Log level (info, debug, warn, error)

### Testing
- `TEST_DATABASE_URL` - PostgreSQL connection string for test database (separate from production)
- `JWT_SECRET` - Secret key for test JWT tokens

**Important**: Tests use a **separate test database** (`iwash_test`) to ensure production data is never affected.

## 📦 Dependencies

Key dependencies:
- `axum` - Web framework
- `sqlx` - Async PostgreSQL driver
- `tokio` - Async runtime
- `jsonwebtoken` - JWT implementation
- `bcrypt` - Password hashing
- `tracing` - Structured logging
- `serde` - Serialization/deserialization

## 🚧 Future Roadmap

- [ ] Bookings module for laundry services
- [ ] Payment integration
- [ ] Order tracking
- [ ] Email notifications
- [ ] Rate limiting middleware
- [ ] API documentation (OpenAPI/Swagger)
- [ ] Database migrations with sqlx-cli
- [ ] Docker deployment

## 📄 License

MIT
