🧺 Project Context: iWash Backend
🚀 Overview

iWash is a modern laundry-booking backend written in Rust using:

🦀 Axum for the web framework

🐘 PostgreSQL + SQLx for async database access

🔐 JWT for authentication

🧠 Tokio runtime for async tasks

The backend will serve a React Native frontend, providing a clean REST API for users to:

Register / log in securely

Create and track laundry bookings

Handle payment and order statuses

📂 Project Structure
```
iwash/
├── src/
│   ├── main.rs              # Binary entry point
│   ├── lib.rs               # Library crate exports
│   ├── config/              # Configuration management
│   │   └── mod.rs
│   ├── db/                  # Database layer
│   │   └── mod.rs
│   ├── models/              # Domain models & DTOs
│   │   ├── mod.rs
│   │   └── user.rs
│   ├── handlers/            # Business logic / request handlers
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── users.rs
│   │   └── health.rs
│   ├── routes/              # Route definitions (thin layer)
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── users.rs
│   │   └── health.rs
│   ├── middleware/          # Request middleware
│   │   ├── mod.rs
│   │   └── auth.rs
│   ├── errors/              # Error handling
│   │   └── mod.rs
│   ├── services/            # Business services layer
│   │   └── mod.rs
│   └── utils/               # Utilities
│       └── mod.rs
├── tests/                   # Integration tests
│   ├── health_check.rs
│   ├── auth_tests.rs
│   └── users_tests.rs
├── sql/                     # Database schemas
│   └── schema.sql
├── Cargo.toml
├── .env
└── README.md
```

⚙️ Key Dependencies
[dependencies]
I have included the key dependencies in Cargo.toml:
You could add more as needed.

🧩 Current Stage

We have successfully built the core backend foundation:

✅ **Phase 1: Core Setup (COMPLETED)**
- Rust project initialized with modular architecture
- Professional folder structure with separation of concerns
- Dependencies configured (Axum, SQLx, dotenvy, JWT, bcrypt, tracing)
- Database connection implemented with connection pooling
- Robust error handling with AppError + IntoResponse
- Comprehensive structured logging with tracing

✅ **Phase 2: Authentication & Users (COMPLETED)**
- User model with email, password_hash, full_name, phone
- JWT-based authentication middleware (24-hour expiry)
- Register endpoint with validation (POST /api/v1/auth/register)
- Login endpoint with bcrypt verification (POST /api/v1/auth/login)
- Complete Users CRUD API (list, get, create, update, delete, me)
- 18 integration tests covering all endpoints (100% passing)
- API versioning (/api/v1 prefix)

---

🎯 **NEXT STEPS: Laundry Booking System**

**Phase 3: Booking Models & Services**
We will now implement the core laundry booking functionality:

1. **Database Schema** (`sql/bookings_schema.sql`)
   - `services` table (wash types: Regular, Delicate, Dry Clean, etc.)
   - `bookings` table (user_id, service_id, status, pickup_address, delivery_address, scheduled_time)
   - `pricing` table (service_id, price_per_kg, base_price)
   - `booking_items` table (booking_id, item_type, quantity, weight_kg)
   - Enums: BookingStatus (Pending, Confirmed, PickedUp, InProgress, Ready, Delivered, Cancelled)

2. **Models** (`src/models/booking.rs`, `src/models/service.rs`)
   ```rust
   // Booking lifecycle
   - CreateBookingRequest
   - UpdateBookingRequest
   - BookingResponse
   - Service (laundry service types)
   - BookingItem (items in a booking)
   ```

3. **Handlers** (`src/handlers/bookings.rs`, `src/handlers/services.rs`)
   - Create booking (authenticated users)
   - List user's bookings (with filters: status, date range)
   - Get booking details
   - Update booking status (admin/staff role)
   - Cancel booking
   - List available services
   - Get service pricing

4. **Routes** (`src/routes/bookings.rs`, `src/routes/services.rs`)
   ```
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
- Invoice generation

**Phase 5: Admin Dashboard (Future)**
- Admin role-based access
- Booking management
- User management
- Analytics & reporting

**Phase 6: Notifications (Future)**
- Email notifications (booking confirmations, status updates)
- SMS notifications
- Push notifications for mobile app


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
- Log important events (user registration, booking creation, etc.)
- Log errors with context
- Use appropriate log levels (info, warn, error)
- Include request IDs for traceability (future enhancement)

Example .env
DATABASE_URL=postgres://postgres:password@localhost/iwash_db
JWT_SECRET=super_secret_key

🧭 Goal

To produce a production-ready Rust backend that can be easily connected to a React Native frontend for a laundry service platform (user bookings, payments, order tracking).

**Development Philosophy:**
- MAKE SURE WE DEVELOP, BUILD AND RUN ONE STEP AT A TIME!
- We have to ensure everything is working properly before moving on to the next step
- Write tests before considering a feature complete
- Build incrementally: Model → Handler → Route → Test → Validate

**Current API Endpoints:**

**Health & Monitoring:**
- GET  /api/v1/health - Health check (public)

**Authentication:**
- POST /api/v1/auth/register - Register new user
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