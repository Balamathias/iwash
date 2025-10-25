-- Multi-Vendor Platform Migration
-- This migration transforms iWash into a multi-vendor laundry booking platform

-- User roles enum
CREATE TYPE user_role AS ENUM ('customer', 'vendor', 'admin');

-- Add role column to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS role user_role NOT NULL DEFAULT 'customer';
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE users ADD COLUMN IF NOT EXISTS email_verified BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE users ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
ALTER TABLE users ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Create index for role-based queries
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_is_active ON users(is_active);

-- Vendors table: Business profiles for laundry service providers
CREATE TABLE IF NOT EXISTS vendors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    business_name VARCHAR(255) NOT NULL,
    business_description TEXT,
    logo_url TEXT,
    banner_url TEXT,
    business_email VARCHAR(255),
    business_phone VARCHAR(20),
    business_address TEXT NOT NULL,
    city VARCHAR(100),
    state VARCHAR(100),
    postal_code VARCHAR(20),
    country VARCHAR(100) DEFAULT 'Nigeria',
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    operating_hours JSONB, -- {"monday": {"open": "08:00", "close": "18:00"}, ...}
    service_radius_km INTEGER DEFAULT 10, -- Service delivery radius
    rating DECIMAL(3, 2) DEFAULT 0.00,
    total_reviews INTEGER DEFAULT 0,
    total_bookings INTEGER DEFAULT 0,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    bank_account_name VARCHAR(255),
    bank_account_number VARCHAR(50),
    bank_name VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for vendor queries
CREATE INDEX IF NOT EXISTS idx_vendors_user_id ON vendors(user_id);
CREATE INDEX IF NOT EXISTS idx_vendors_city ON vendors(city);
CREATE INDEX IF NOT EXISTS idx_vendors_is_active ON vendors(is_active);
CREATE INDEX IF NOT EXISTS idx_vendors_rating ON vendors(rating DESC);
CREATE INDEX IF NOT EXISTS idx_vendors_location ON vendors(latitude, longitude);

-- Add vendor_id to services table
ALTER TABLE services ADD COLUMN IF NOT EXISTS vendor_id UUID REFERENCES vendors(id) ON DELETE CASCADE;
ALTER TABLE services ADD COLUMN IF NOT EXISTS is_featured BOOLEAN NOT NULL DEFAULT false;

-- Update services index
CREATE INDEX IF NOT EXISTS idx_services_vendor_id ON services(vendor_id);
CREATE INDEX IF NOT EXISTS idx_services_featured ON services(is_featured) WHERE is_featured = true;

-- Reviews table: Customer reviews for vendors
CREATE TABLE IF NOT EXISTS reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL REFERENCES vendors(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    booking_id UUID REFERENCES bookings(id) ON DELETE SET NULL,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    vendor_response TEXT,
    vendor_response_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(booking_id) -- One review per booking
);

-- Indexes for reviews
CREATE INDEX IF NOT EXISTS idx_reviews_vendor_id ON reviews(vendor_id);
CREATE INDEX IF NOT EXISTS idx_reviews_user_id ON reviews(user_id);
CREATE INDEX IF NOT EXISTS idx_reviews_booking_id ON reviews(booking_id);
CREATE INDEX IF NOT EXISTS idx_reviews_rating ON reviews(rating);

-- Vendor service areas table (for location-based search)
CREATE TABLE IF NOT EXISTS vendor_service_areas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL REFERENCES vendors(id) ON DELETE CASCADE,
    area_name VARCHAR(255) NOT NULL,
    city VARCHAR(100),
    state VARCHAR(100),
    postal_codes TEXT[], -- Array of postal codes served
    delivery_fee_cents INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for service areas
CREATE INDEX IF NOT EXISTS idx_vendor_service_areas_vendor_id ON vendor_service_areas(vendor_id);
CREATE INDEX IF NOT EXISTS idx_vendor_service_areas_city ON vendor_service_areas(city);

-- Vendor availability/schedule table
CREATE TABLE IF NOT EXISTS vendor_availability (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL REFERENCES vendors(id) ON DELETE CASCADE,
    day_of_week INTEGER NOT NULL CHECK (day_of_week >= 0 AND day_of_week <= 6), -- 0=Sunday, 6=Saturday
    open_time TIME NOT NULL,
    close_time TIME NOT NULL,
    is_available BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(vendor_id, day_of_week)
);

-- Index for availability
CREATE INDEX IF NOT EXISTS idx_vendor_availability_vendor_id ON vendor_availability(vendor_id);

-- Update existing default services to be vendor-owned
-- For now, create a default "iWash Platform" vendor for existing services
DO $$
DECLARE
    platform_user_id UUID;
    platform_vendor_id UUID;
BEGIN
    -- Check if platform user exists, if not create one
    INSERT INTO users (id, email, password_hash, full_name, role)
    VALUES (
        gen_random_uuid(),
        'platform@iwash.com',
        '$2b$12$dummy_hash_for_platform_user',
        'iWash Platform',
        'admin'
    )
    ON CONFLICT (email) DO NOTHING
    RETURNING id INTO platform_user_id;

    -- Get the platform user id if it already existed
    IF platform_user_id IS NULL THEN
        SELECT id INTO platform_user_id FROM users WHERE email = 'platform@iwash.com';
    END IF;

    -- Create platform vendor if doesn't exist
    INSERT INTO vendors (id, user_id, business_name, business_description, business_address, city, state, country, is_verified, is_active)
    VALUES (
        gen_random_uuid(),
        platform_user_id,
        'iWash Platform Services',
        'Official iWash platform laundry services',
        'Main Street, Victoria Island',
        'Lagos',
        'Lagos State',
        'Nigeria',
        true,
        true
    )
    ON CONFLICT (user_id) DO NOTHING
    RETURNING id INTO platform_vendor_id;

    -- Get the platform vendor id if it already existed
    IF platform_vendor_id IS NULL THEN
        SELECT id INTO platform_vendor_id FROM vendors WHERE user_id = platform_user_id;
    END IF;

    -- Update existing services to belong to platform vendor
    UPDATE services SET vendor_id = platform_vendor_id WHERE vendor_id IS NULL;
END $$;

-- Add notification preferences table
CREATE TABLE IF NOT EXISTS notification_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    email_notifications BOOLEAN NOT NULL DEFAULT true,
    sms_notifications BOOLEAN NOT NULL DEFAULT false,
    push_notifications BOOLEAN NOT NULL DEFAULT true,
    booking_updates BOOLEAN NOT NULL DEFAULT true,
    marketing_emails BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for notifications
CREATE INDEX IF NOT EXISTS idx_notification_preferences_user_id ON notification_preferences(user_id);

-- Vendor payout tracking table (for future payment integration)
CREATE TABLE IF NOT EXISTS vendor_payouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_id UUID NOT NULL REFERENCES vendors(id) ON DELETE CASCADE,
    amount_cents INTEGER NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, processing, completed, failed
    booking_ids UUID[] NOT NULL,
    payout_method VARCHAR(50), -- bank_transfer, mobile_money, etc.
    reference_number VARCHAR(255),
    notes TEXT,
    processed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for payouts
CREATE INDEX IF NOT EXISTS idx_vendor_payouts_vendor_id ON vendor_payouts(vendor_id);
CREATE INDEX IF NOT EXISTS idx_vendor_payouts_status ON vendor_payouts(status);
CREATE INDEX IF NOT EXISTS idx_vendor_payouts_created_at ON vendor_payouts(created_at DESC);

COMMENT ON TABLE vendors IS 'Vendor business profiles for multi-vendor laundry platform';
COMMENT ON TABLE reviews IS 'Customer reviews and ratings for vendors';
COMMENT ON TABLE vendor_service_areas IS 'Geographic areas served by each vendor';
COMMENT ON TABLE vendor_availability IS 'Vendor operating hours by day of week';
COMMENT ON TABLE vendor_payouts IS 'Vendor earnings and payout tracking';
COMMENT ON COLUMN users.role IS 'User role: customer, vendor, or admin';
COMMENT ON COLUMN vendors.service_radius_km IS 'Maximum delivery radius from vendor location';
COMMENT ON COLUMN vendors.rating IS 'Average vendor rating (0.00 to 5.00)';
