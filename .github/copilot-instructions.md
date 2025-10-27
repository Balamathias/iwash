🧺 Project Context: iWash Backend
🚀 Overview

iWash is a modern **multi-vendor laundry booking platform** written in Rust using:

🦀 Axum for the web framework

🐘 PostgreSQL + SQLx for async database access

🔐 JWT for authentication with role-based access control

🧠 Tokio runtime for async tasks

The backend serves a React Native frontend, providing a clean REST API for:

**Customers** to:
- Browse and search vendors by location and rating
- Book laundry services from different vendors
- Track order status in real-time
- Leave reviews and ratings

**Vendors** to:
- Register and manage their laundry business
- Create and price their services
- Manage bookings and update statuses
- Track earnings and request payouts
- Respond to customer reviews

**Admins** to:
- Verify vendor businesses
- Oversee platform operations
- Manage disputes
- View analytics

📂 Project Structure
```
iwash/
├── src/
│   ├── main.rs              # Binary entry point with middleware stack
│   ├── lib.rs               # Library crate exports
│   ├── config/              # Configuration management
│   │   └── mod.rs
│   ├── db/                  # Database layer
│   │   └── mod.rs
│   ├── models/              # Domain models & DTOs
│   │   ├── mod.rs
│   │   ├── user.rs          # User, UserRole, UserResponse
│   │   ├── vendor.rs        # Vendor, Review models
│   │   ├── service.rs       # Service model  
│   │   └── booking.rs       # Booking, BookingItem, BookingStatus
│   ├── handlers/            # Business logic / request handlers
│   │   ├── mod.rs
│   │   ├── auth.rs          # Registration, login
│   │   ├── users.rs         # User CRUD
│   │   ├── vendors.rs       # Vendor management
│   │   ├── services.rs      # Service listing & vendor services
│   │   ├── bookings.rs      # Booking management
│   │   └── health.rs        # Health checks
│   ├── routes/              # Route definitions (thin layer)
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── users.rs
│   │   ├── vendors.rs
│   │   ├── services.rs
│   │   ├── bookings.rs
│   │   └── health.rs
│   ├── middleware/          # Request middleware
│   │   ├── mod.rs
│   │   ├── auth.rs          # JWT authentication
│   │   └── request_id.rs    # Request ID tracking
│   ├── errors/              # Error handling
│   │   └── mod.rs           # AppError with error codes
│   ├── services/            # Business services layer
│   │   └── mod.rs
│   └── utils/               # Utilities
│       └── mod.rs
├── tests/                   # Integration tests
│   ├── common/
│   │   └── mod.rs           # Test utilities
│   ├── health_check.rs      # 1 test
│   ├── auth_tests.rs        # 6 tests
│   ├── users_tests.rs       # 15 tests
│   ├── bookings_tests.rs    # 9 tests
│   └── vendors_tests.rs     # 16 tests (vendor services tests)
├── migrations/              # sqlx-cli versioned migrations (industry standard)
│   ├── 20251027212411_initial_schema.up.sql
│   ├── 20251027212411_initial_schema.down.sql
│   ├── 20251027212426_bookings_schema.up.sql
│   ├── 20251027212426_bookings_schema.down.sql
│   ├── 20251027212437_multi_vendor.up.sql
│   └── 20251027212437_multi_vendor.down.sql
├── sql/                     # Legacy SQL files (DEPRECATED - use migrations/)
│   ├── 001_schema.sql       # Migrated to migrations/
│   ├── 002_bookings_schema.sql  # Migrated to migrations/
│   └── 003_multi_vendor_migration.sql  # Migrated to migrations/
├── Cargo.toml
├── .env
└── README.md
```

⚙️ Key Dependencies
[dependencies]
I have included the key dependencies in Cargo.toml:
You could add more as needed.

🧩 Current Stage

We have successfully built a robust multi-vendor laundry booking platform:

✅ **Phase 1: Core Setup (COMPLETED)**
- Rust project initialized with modular architecture
- Professional folder structure with separation of concerns
- Dependencies configured (Axum, SQLx, JWT, bcrypt, tracing, tower-http)
- Database connection implemented with connection pooling
- Robust error handling with AppError + IntoResponse with error codes
- Comprehensive structured logging with tracing
- HTTP request/response logging with TraceLayer
- Rate limiting (100 requests burst, 2/sec sustained)
- CORS support for React Native frontend
- Request ID tracking middleware

✅ **Phase 2: Authentication & Users (COMPLETED)**
- User model with full_name, phone, role (customer/vendor/admin)
- JWT-based authentication middleware (24-hour expiry)
- Register endpoint with role selection (POST /api/v1/auth/register)
- Login endpoint with bcrypt verification (POST /api/v1/auth/login)
- Complete Users CRUD API (list, get, create, update, delete, me)
- Pagination support (page, limit up to 100)
- Search functionality (by email and name)
- 22 integration tests covering all endpoints (100% passing)
- API versioning (/api/v1 prefix)

✅ **Phase 3: Multi-Vendor Architecture (COMPLETED)**
- UserRole enum (Customer, Vendor, Admin)
- Vendors table with comprehensive business profiles
- Reviews and ratings system
- Vendor service areas (location-based)
- Vendor availability schedules
- Vendor payout tracking
- Services linked to vendors (vendor_id FK)
- Booking system with status tracking
- Database migrations applied to production and test databases

---

🎯 **CURRENT WORK: Multi-Vendor Implementation**

**Phase 4: Vendor Management & Public APIs (COMPLETED)**

All vendor management features are fully implemented with role-based access control:

✅ **Vendor Profile Management**
   - Create vendor profile (POST /api/v1/vendors) - Vendor role required
   - List all vendors (GET /api/v1/vendors) - PUBLIC, with filters
   - Get vendor details (GET /api/v1/vendors/{id}) - PUBLIC
   - Get own vendor profile (GET /api/v1/vendors/me) - Vendor role
   - Update vendor profile (PATCH /api/v1/vendors/{id}) - Owner only
   - Get vendor statistics (GET /api/v1/vendors/me/stats) - Vendor role

✅ **Vendor Service Management**
   - Create service (POST /api/v1/vendors/me/services) - Vendor role required
   - List own services (GET /api/v1/vendors/me/services) - Vendor role
   - Update service (PATCH /api/v1/vendors/me/services/{id}) - Owner only
   - Delete/deactivate service (DELETE /api/v1/vendors/me/services/{id}) - Owner only
   - List vendor's services PUBLIC (GET /api/v1/vendors/{id}/services) - No auth required

✅ **Reviews & Ratings**
   - List vendor reviews PUBLIC (GET /api/v1/vendors/{id}/reviews) - No auth required

✅ **Booking Management for Vendors**
   - List vendor's bookings (GET /api/v1/bookings/vendor) - Vendor role required
   - Update booking status (PATCH /api/v1/bookings/vendor/{id}/status) - Vendor role
   - Automatic price calculation: total_price = base_price + (price_per_kg × weight)

✅ **Role-Based Access Control**
   - RequireVendor middleware for vendor-only endpoints
   - RequireAdmin middleware for admin-only endpoints
   - Custom error messages with AppError::Forbidden(Option<String>)

2. **API Endpoints Summary**
   
   **Public Endpoints** (No authentication):
   ```
   GET    /api/v1/vendors                - List all vendors with filters
   GET    /api/v1/vendors/{id}           - Get vendor details
   GET    /api/v1/vendors/{id}/services  - List vendor's services
   GET    /api/v1/vendors/{id}/reviews   - List vendor's reviews
   GET    /api/v1/services               - List all active services
   GET    /api/v1/services/{id}          - Get service details
   ```

   **Vendor-Only Endpoints** (Vendor role required):
   ```
   POST   /api/v1/vendors                - Create vendor profile
   GET    /api/v1/vendors/me             - Get own vendor profile
   PATCH  /api/v1/vendors/{id}           - Update vendor profile (owner only)
   GET    /api/v1/vendors/me/stats       - Get vendor dashboard statistics
   
   POST   /api/v1/vendors/me/services    - Create new service
   GET    /api/v1/vendors/me/services    - List own services
   PATCH  /api/v1/vendors/me/services/{id} - Update service (owner only)
   DELETE /api/v1/vendors/me/services/{id} - Deactivate service (owner only)
   
   GET    /api/v1/bookings/vendor        - List vendor's bookings
   PATCH  /api/v1/bookings/vendor/{id}/status - Update booking status
   ```

3. **Key Features Implemented**

3. **Key Features Implemented**
   - Automatic price calculation for bookings (base_price + price_per_kg × weight)
   - Service ownership validation (vendors can only modify their own services)
   - Soft delete for services (is_active = false)
   - Public vendor discovery with filters (city, rating, verification status)
   - Vendor dashboard with statistics (bookings, revenue, ratings)
   - Review system for vendor feedback
   - Role-based access control with clear error messages

---

🎯 **NEXT STEPS: Additional Features**

**Phase 5: Reviews & Ratings (Future)**
- Customer can leave review after booking completion
- Vendor can respond to reviews
- Calculate and update vendor average rating
- List reviews for a vendor (paginated)

**Phase 6: Payments & Payouts (Future)**
- Payment integration (Paystack/Flutterwave)
- Booking payment processing
- Vendor earnings tracking
- Payout request and processing
- Invoice generation

**Phase 7: Notifications (Future)**
- Email notifications (booking confirmations, status updates)
- SMS notifications for critical updates
- Push notifications for mobile app
- Vendor notification preferences

**Phase 8: Admin Dashboard (Future)**
- Admin endpoints for vendor verification
- Platform analytics and metrics
- Dispute management
- User and vendor moderation
- System configuration


When assisting in this project, the AI agent should:
   POST   /api/v1/bookings          - Create new booking
   GET    /api/v1/bookings          - List user's bookings
   GET    /api/v1/bookings/:id      - Get booking details
   PATCH  /api/v1/bookings/:id      - Update booking
   DELETE /api/v1/bookings/:id      - Cancel booking
   
   GET    /api/v1/services          - List laundry services
   GET    /api/v1/services/:id      - Get service details
   ```

5. **Business Logic**
   - Validate pickup/delivery addresses
   - Calculate pricing based on weight and service type
   - Prevent double-booking for same time slot
   - Send booking confirmations (future: email/SMS)
   - Track booking status transitions

6. **Tests** (`tests/bookings_tests.rs`)
   - Create booking success
   - List bookings with filters
   - Update booking status
   - Cancel booking
   - Invalid booking scenarios

**Phase 4: Payments (Future)**
- Payment model (booking_id, amount, status, payment_method)
- Integration with payment gateway (Stripe/PayPal)
- Payment confirmation webhooks


When assisting in this project, the AI agent should:

**Code Quality & Standards:**
- Maintain idiomatic Rust patterns (ownership, lifetimes, error handling)
- Use async/await correctly with tokio
- Keep code modular (separate routes, handlers, models, services)
- Follow RESTful API best practices
- Output complete, compilable snippets
- Optimize for clarity and maintainability (commented examples)
- Use professional error handling with AppError and proper HTTP status codes

**Architecture Principles:**
- Follow separation of concerns (handlers for logic, routes for HTTP wiring)
- Keep handlers thin, move complex logic to services layer when needed
- Use DTOs (Data Transfer Objects) for request/response validation
- Normalize data before database operations
- Always use transactions for multi-step database operations

**Security Best Practices:**
- Always validate input (emails, phone numbers, addresses, etc.)
- Use JWT authentication for protected endpoints
- Hash passwords with bcrypt (DEFAULT_COST)
- Never expose sensitive data (password_hash) in API responses
- Implement role-based access control where needed
- Sanitize SQL inputs (SQLx handles this via prepared statements)

**Testing Requirements:**
- Write integration tests for all new endpoints
- Use unique identifiers (timestamps) to avoid test conflicts
- Test both success and error cases
- Verify HTTP status codes and response bodies
- Maintain >90% test coverage

**Database:**
- Assume PostgreSQL is used locally via .env
- Use SQLx for all database operations (compile-time verified queries)
- Generate UUIDs in application code (not database)
- Use COALESCE for partial updates
- Add proper indexes for frequently queried fields

**API Design:**
- All endpoints under /api/v1 prefix
- Use proper HTTP methods (GET, POST, PATCH, DELETE)
- Return consistent JSON error responses
- Include pagination for list endpoints (limit/offset)
- Use proper status codes (200, 201, 400, 401, 404, 500)

**Logging:**
- Use tracing crate for structured logging
- Log important events (user registration, booking creation, vendor registration, etc.)
- Log errors with context
- Use appropriate log levels (info, warn, error)
- Include request IDs for traceability via request_id middleware

**Database Migrations:**
- iWash uses sqlx-cli for industry-standard versioned migrations
- All schema changes must be in timestamped migration files in `migrations/` folder
- Never modify existing migration files after they've been applied to production
- Always create both `.up.sql` and `.down.sql` files for reversibility
- Test migrations on a copy of production data before deploying
- Migration tracking is automatic via `_sqlx_migrations` table
- Run migrations: `sqlx migrate run --database-url $DATABASE_URL`
- Create new migration: `sqlx migrate add -r migration_name`
- Cloud deployments: Use `sslmode=require` in DATABASE_URL for SSL/TLS

Example .env
```env
DATABASE_URL=postgres://postgres:password@localhost/iwash_db
TEST_DATABASE_URL=postgres://postgres:password@localhost/iwash_test
JWT_SECRET=super_secret_key_change_in_production
```

Example .env.production (Cloud)
```env
DATABASE_URL=postgres://username:password@aws-rds-host.com:5432/iwash_production?sslmode=require
JWT_SECRET=your_production_jwt_secret_256_bits
```

🧭 Goal

To produce a production-ready Rust backend for a **multi-vendor laundry booking platform** that can be easily connected to a React Native frontend, supporting customers, vendors, and admins.

**Development Philosophy:**
- MAKE SURE WE DEVELOP, BUILD AND RUN ONE STEP AT A TIME!
- We have to ensure everything is working properly before moving on to the next step
- Write tests before considering a feature complete
- Build incrementally: Model → Handler → Route → Test → Validate

**Current API Endpoints:**

**Health & Monitoring:**
- GET  /api/v1/health - Health check with database status (public)

**Authentication:**
- POST /api/v1/auth/register - Register new user (customer/vendor/admin)
- POST /api/v1/auth/login - Login and get JWT token

**Users (Protected):**
- GET    /api/v1/users/me - Get current user profile
- GET    /api/v1/users - List all users
- POST   /api/v1/users - Create a new user
- GET    /api/v1/users/{id} - Get user by ID
- PATCH  /api/v1/users/{id} - Update user
- DELETE /api/v1/users/{id} - Delete user

**Test Coverage:**
- 18 integration tests (100% passing)
- Auth tests: 6
- Users tests: 11
- Health check: 1
**Services (Public):**
- GET    /api/v1/services - List all active services
- GET    /api/v1/services/{id} - Get service details

**Bookings (Protected):**
- POST   /api/v1/bookings - Create new booking
- GET    /api/v1/bookings - List user's bookings (paginated, filterable)
- GET    /api/v1/bookings/{id} - Get booking details
- PATCH  /api/v1/bookings/{id} - Update booking
- DELETE /api/v1/bookings/{id}/cancel - Cancel booking

**Vendors (TBD - Next Phase):**
- POST   /api/v1/vendors - Create vendor profile (Vendor role)
- GET    /api/v1/vendors - List all vendors (PUBLIC, filterable)
- GET    /api/v1/vendors/me - Get own vendor profile (Vendor role)
- GET    /api/v1/vendors/{id} - Get vendor details (PUBLIC)
- PATCH  /api/v1/vendors/{id} - Update vendor profile (Vendor role)
- GET    /api/v1/vendors/{id}/services - List vendor services (PUBLIC)
- GET    /api/v1/vendors/{id}/reviews - List vendor reviews (PUBLIC)
