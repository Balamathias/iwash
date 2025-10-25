# iWash API Quick Reference

**Base URL**: `http://localhost:3000/api/v1`

---

## 🔓 Public Endpoints (No Authentication Required)

### Health Check
```http
GET /health
```
**Response**: `200 OK`
```json
{
  "status": "ok",
  "database": "connected"
}
```

### Browse Vendors
```http
GET /vendors?city=Lagos&search=laundry&min_rating=4.0&verified=true&page=1&limit=20
```
**Query Parameters**:
- `city` (optional) - Filter by city name
- `search` (optional) - Search business name/description
- `min_rating` (optional) - Minimum rating (0.0-5.0)
- `verified` (optional) - true/false
- `page` (optional) - Page number (default: 1)
- `limit` (optional) - Items per page (default: 10, max: 100)

**Response**: `200 OK`
```json
{
  "vendors": [
    {
      "id": "uuid",
      "user_id": "uuid",
      "business_name": "Clean & Shine Laundry",
      "business_description": "Professional laundry services",
      "business_email": "contact@cleanshine.com",
      "business_phone": "+2341234567890",
      "business_address": "123 Main Street",
      "city": "Lagos",
      "state": "Lagos",
      "country": "Nigeria",
      "rating": 4.5,
      "total_reviews": 128,
      "total_bookings": 256,
      "is_verified": true,
      "is_active": true,
      "created_at": "2025-01-15T10:30:00Z"
    }
  ],
  "total": 45,
  "page": 1,
  "limit": 20
}
```

### Get Vendor Details
```http
GET /vendors/{vendor_id}
```
**Response**: `200 OK` (same structure as vendor in list)

### Browse Services
```http
GET /services?vendor_id=uuid&page=1&limit=20
```
**Query Parameters**:
- `vendor_id` (optional) - Filter by vendor
- `page` (optional)
- `limit` (optional)

**Response**: `200 OK`
```json
{
  "services": [
    {
      "id": "uuid",
      "vendor_id": "uuid",
      "name": "Wash & Fold",
      "description": "Regular laundry service",
      "base_price_cents": 1000,
      "price_per_kg_cents": 500,
      "is_featured": true,
      "is_active": true
    }
  ],
  "total": 12,
  "page": 1,
  "limit": 20
}
```

### Get Service Details
```http
GET /services/{service_id}
```
**Response**: `200 OK` (same structure as service in list)

---

## 🔐 Authentication

### Register
```http
POST /auth/register
Content-Type: application/json

{
  "full_name": "John Doe",
  "email": "john@example.com",
  "phone": "+2341234567890",
  "password": "securepassword123",
  "role": "customer"  // or "vendor" or "admin"
}
```
**Response**: `201 Created`
```json
{
  "user": {
    "id": "uuid",
    "full_name": "John Doe",
    "email": "john@example.com",
    "phone": "+2341234567890",
    "role": "customer",
    "is_active": true,
    "email_verified": false,
    "created_at": "2025-10-25T12:00:00Z"
  },
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### Login
```http
POST /auth/login
Content-Type: application/json

{
  "email": "john@example.com",
  "password": "securepassword123"
}
```
**Response**: `200 OK` (same structure as register)

---

## 👤 User Management (Authenticated)

**Authorization**: `Bearer {token}`

### Get My Profile
```http
GET /users/me
Authorization: Bearer {token}
```

### List Users
```http
GET /users?search=john&page=1&limit=20
Authorization: Bearer {token}
```

### Get User by ID
```http
GET /users/{user_id}
Authorization: Bearer {token}
```

### Update User
```http
PATCH /users/{user_id}
Authorization: Bearer {token}
Content-Type: application/json

{
  "full_name": "John Updated",
  "phone": "+2349876543210"
}
```

### Delete User
```http
DELETE /users/{user_id}
Authorization: Bearer {token}
```

---

## 🏪 Vendor Management

### Create Vendor Profile (Vendor Role Required)
```http
POST /vendors
Authorization: Bearer {token}
Content-Type: application/json

{
  "business_name": "Clean & Shine Laundry",
  "business_description": "Professional laundry services since 2020",
  "business_email": "contact@cleanshine.com",
  "business_phone": "+2341234567890",
  "business_address": "123 Main Street, Ikeja",
  "city": "Lagos",
  "state": "Lagos",
  "postal_code": "100001",
  "country": "Nigeria",
  "latitude": 6.5244,
  "longitude": 3.3792,
  "service_radius_km": 10
}
```
**Response**: `201 Created`

### Get My Vendor Profile
```http
GET /vendors/me
Authorization: Bearer {token}
```

### Update Vendor Profile
```http
PATCH /vendors/{vendor_id}
Authorization: Bearer {token}
Content-Type: application/json

{
  "business_name": "Updated Name",
  "business_description": "New description",
  "logo_url": "https://example.com/logo.png"
}
```
**Response**: `200 OK`

### Get Vendor Dashboard Stats
```http
GET /vendors/me/stats
Authorization: Bearer {token}
```
**Response**: `200 OK`
```json
{
  "vendor_id": "uuid",
  "business_name": "Clean & Shine",
  "rating": 4.5,
  "total_reviews": 128,
  "total_bookings": 256,
  "pending_bookings": 12,
  "completed_bookings": 200,
  "total_revenue_cents": 1500000,
  "is_verified": true,
  "is_active": true
}
```

---

## 📦 Bookings

### Customer Endpoints

#### Create Booking
```http
POST /bookings
Authorization: Bearer {token}
Content-Type: application/json

{
  "service_id": "uuid",
  "pickup_address": "456 Customer Street",
  "delivery_address": "456 Customer Street",
  "pickup_time": "2025-10-26T09:00:00Z",
  "delivery_time": "2025-10-27T18:00:00Z",
  "estimated_weight_kg": 5.0,
  "special_instructions": "Handle with care"
}
```
**Response**: `201 Created`
```json
{
  "id": "uuid",
  "user_id": "uuid",
  "service_id": "uuid",
  "vendor": {
    "id": "uuid",
    "business_name": "Clean & Shine",
    "business_phone": "+2341234567890",
    "city": "Lagos",
    "rating": 4.5
  },
  "status": "pending",
  "pickup_address": "456 Customer Street",
  "delivery_address": "456 Customer Street",
  "pickup_time": "2025-10-26T09:00:00Z",
  "delivery_time": "2025-10-27T18:00:00Z",
  "estimated_weight_kg": 5.0,
  "actual_weight_kg": null,
  "total_price_cents": null,
  "special_instructions": "Handle with care",
  "created_at": "2025-10-25T12:00:00Z",
  "updated_at": "2025-10-25T12:00:00Z"
}
```

#### List My Bookings
```http
GET /bookings?status=pending&from_date=2025-10-01&to_date=2025-10-31&page=1&limit=20
Authorization: Bearer {token}
```
**Query Parameters**:
- `status` - pending, confirmed, in_progress, completed, cancelled
- `from_date` - ISO 8601 date
- `to_date` - ISO 8601 date
- `page`, `limit`

#### Get Booking Details
```http
GET /bookings/{booking_id}
Authorization: Bearer {token}
```

#### Update Booking
```http
PATCH /bookings/{booking_id}
Authorization: Bearer {token}
Content-Type: application/json

{
  "pickup_address": "New address",
  "special_instructions": "Updated instructions"
}
```

#### Cancel Booking
```http
DELETE /bookings/{booking_id}/cancel
Authorization: Bearer {token}
```

### Vendor Endpoints

#### List Vendor's Bookings
```http
GET /bookings/vendor?status=pending&from_date=2025-10-01&page=1
Authorization: Bearer {token}
```

#### Get Vendor Booking Details
```http
GET /bookings/vendor/{booking_id}
Authorization: Bearer {token}
```

#### Update Booking Status
```http
PATCH /bookings/vendor/{booking_id}/status
Authorization: Bearer {token}
Content-Type: application/json

{
  "status": "confirmed",
  "actual_weight_kg": 5.5,
  "total_price_cents": 2750
}
```
**Allowed Status Transitions**:
- pending → confirmed
- confirmed → in_progress
- in_progress → completed
- any → cancelled

---

## 🚫 Error Responses

### 400 Bad Request
```json
{
  "error": "Invalid request data",
  "code": "BAD_REQUEST"
}
```

### 401 Unauthorized
```json
{
  "error": "Missing or invalid authentication token",
  "code": "UNAUTHORIZED"
}
```

### 403 Forbidden
```json
{
  "error": "You don't have permission to access this resource",
  "code": "FORBIDDEN"
}
```

### 404 Not Found
```json
{
  "error": "Resource not found",
  "code": "NOT_FOUND"
}
```

### 500 Internal Server Error
```json
{
  "error": "An internal error occurred",
  "code": "INTERNAL_ERROR"
}
```

---

## 📝 Notes

### Authentication
- Include JWT token in `Authorization: Bearer {token}` header
- Tokens expire after 24 hours
- Register with appropriate role (customer/vendor/admin)

### Pagination
- Default page: 1
- Default limit: 10
- Maximum limit: 100

### Timestamps
- All timestamps are in ISO 8601 format (UTC)
- Example: `2025-10-25T12:00:00Z`

### Money
- All prices in cents (e.g., $10.00 = 1000 cents)
- Prevents floating-point rounding errors

### Coordinates
- Latitude: -90.0 to 90.0
- Longitude: -180.0 to 180.0

### Role-Based Access
- **Customer**: Can browse, book, manage own bookings
- **Vendor**: Customer permissions + vendor profile + vendor bookings
- **Admin**: All permissions (future feature)

---

## 🧪 Testing with cURL

### Register as Vendor
```bash
curl -X POST http://localhost:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "full_name": "Vendor User",
    "email": "vendor@test.com",
    "phone": "+2341234567890",
    "password": "password123",
    "role": "vendor"
  }'
```

### Create Vendor Profile
```bash
curl -X POST http://localhost:3000/api/v1/vendors \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "business_name": "My Laundry",
    "business_address": "123 Main St",
    "city": "Lagos"
  }'
```

### Browse Vendors (Public)
```bash
curl http://localhost:3000/api/v1/vendors?city=Lagos
```

### Create Booking
```bash
curl -X POST http://localhost:3000/api/v1/bookings \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "service_id": "SERVICE_UUID",
    "pickup_address": "456 Customer St",
    "delivery_address": "456 Customer St",
    "pickup_time": "2025-10-26T09:00:00Z",
    "delivery_time": "2025-10-27T18:00:00Z",
    "estimated_weight_kg": 5.0
  }'
```

---

**Version**: 1.0.0  
**Last Updated**: October 25, 2025
