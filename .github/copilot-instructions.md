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
│   │   ├── vendors.rs       # Vendor management (TBD)
│   │   ├── services.rs      # Service listing
│   │   ├── bookings.rs      # Booking management
│   │   └── health.rs        # Health checks
│   ├── routes/              # Route definitions (thin layer)
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── users.rs
│   │   ├── vendors.rs       # (TBD)
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
│   ├── bookings_tests.rs    # (TBD)
│   └── vendors_tests.rs     # (TBD)
├── sql/                     # Database schemas
│   ├── schema.sql           # Base schema
│   ├── test_schema.sql      # Test database schema
│   ├── bookings_schema.sql  # Booking system
│   └── multi_vendor_migration.sql  # Multi-vendor architecture
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
Vendor management handlers, routes and vendor service management (create/update/delete) are implemented. Booking creation now calculates prices automatically from service pricing. The following endpoints are available:

1. **Vendor Handlers** (`src/handlers/vendors.rs`)
   - Register as vendor (create vendor profile)
   - Get vendor profile (own and public)
   - Update vendor profile
   - List all vendors (PUBLIC, filterable by city, rating, verified status)
   - Get vendor details (PUBLIC)
   - Vendor dashboard stats

2. **Vendor Routes** (`src/routes/vendors.rs`)
   ```
   POST   /api/v1/vendors          - Create vendor profile (Vendor role required)
   GET    /api/v1/vendors          - List all vendors (PUBLIC)
   GET    /api/v1/vendors/me       - Get own vendor profile (Vendor role)
   GET    /api/v1/vendors/{id}     - Get vendor details (PUBLIC)
   PATCH  /api/v1/vendors/{id}     - Update vendor profile (Vendor role)
   GET    /api/v1/vendors/{id}/services  - List vendor services (PUBLIC)
   GET    /api/v1/vendors/{id}/reviews   - List vendor reviews (PUBLIC)
   ```

3. **Role-Based Access Control**
   - Update AuthUser middleware to extract user role
   - Create RequireRole middleware (RequireVendor, RequireAdmin)
   - Apply role guards to protected endpoints
   - Make public endpoints accessible without auth

4. **Public Endpoints** (No authentication required)
   - GET /api/v1/vendors (browse all vendors)
   - GET /api/v1/vendors/{id} (vendor details)
   - GET /api/v1/vendors/{id}/services (vendor's services)
   - GET /api/v1/vendors/{id}/reviews (vendor reviews)
   - GET /api/v1/services (browse all services)
   - GET /api/v1/services/{id} (service details)

5. **Update Services Handlers**
   - Make list_services and get_service PUBLIC
   - Add vendor_id filter to list_services
   - Return vendor info with service details

6. **Update Bookings for Multi-Vendor**
   - Link bookings to vendors through services
   - Add vendor_id to booking responses
   - Vendor dashboard: view their bookings
   - Customer view: see vendor info in bookings
   - Vendor can update booking status

---

🎯 **NEXT STEPS: Complete Multi-Vendor Features**

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

Example .env
```env
DATABASE_URL=postgres://postgres:password@localhost/iwash_db
TEST_DATABASE_URL=postgres://postgres:password@localhost/iwash_test
JWT_SECRET=super_secret_key_change_in_production
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
