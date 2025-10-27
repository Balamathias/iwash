-- Booking System Schema for iWash
-- This schema supports laundry services, bookings, pricing, and booking items

-- Services table: Different types of laundry services
CREATE TABLE IF NOT EXISTS services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    base_price_cents INTEGER NOT NULL DEFAULT 0,
    price_per_kg_cents INTEGER NOT NULL DEFAULT 0,
    estimated_duration_hours INTEGER NOT NULL DEFAULT 24,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Booking status enum type
CREATE TYPE booking_status AS ENUM (
    'pending',
    'confirmed',
    'picked_up',
    'in_progress',
    'ready',
    'delivered',
    'cancelled'
);

-- Bookings table: Customer laundry bookings
CREATE TABLE IF NOT EXISTS bookings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    service_id UUID NOT NULL REFERENCES services(id),
    status booking_status NOT NULL DEFAULT 'pending',
    pickup_address TEXT NOT NULL,
    delivery_address TEXT NOT NULL,
    scheduled_pickup_time TIMESTAMPTZ NOT NULL,
    scheduled_delivery_time TIMESTAMPTZ,
    actual_pickup_time TIMESTAMPTZ,
    actual_delivery_time TIMESTAMPTZ,
    total_weight_kg DECIMAL(10, 2),
    total_price_cents INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Booking items: Individual items in a booking (e.g., shirts, pants, etc.)
CREATE TABLE IF NOT EXISTS booking_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    booking_id UUID NOT NULL REFERENCES bookings(id) ON DELETE CASCADE,
    item_type VARCHAR(100) NOT NULL, -- e.g., "shirt", "pants", "dress", "bedsheet"
    quantity INTEGER NOT NULL DEFAULT 1,
    weight_kg DECIMAL(10, 2),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_bookings_user_id ON bookings(user_id);
CREATE INDEX IF NOT EXISTS idx_bookings_status ON bookings(status);
CREATE INDEX IF NOT EXISTS idx_bookings_scheduled_pickup ON bookings(scheduled_pickup_time);
CREATE INDEX IF NOT EXISTS idx_booking_items_booking_id ON booking_items(booking_id);
CREATE INDEX IF NOT EXISTS idx_services_is_active ON services(is_active);

-- Insert default laundry services
INSERT INTO services (name, description, base_price_cents, price_per_kg_cents, estimated_duration_hours) VALUES
    ('Regular Wash', 'Standard wash and dry service for everyday clothes', 500, 300, 24),
    ('Delicate Wash', 'Gentle wash for delicate fabrics like silk and wool', 800, 500, 36),
    ('Dry Clean', 'Professional dry cleaning for suits and formal wear', 1500, 800, 48),
    ('Express Wash', 'Fast wash and dry service completed in 12 hours', 1000, 400, 12),
    ('Iron Only', 'Professional ironing service', 300, 200, 6),
    ('Wash & Iron', 'Complete wash, dry, and iron service', 700, 400, 30);

