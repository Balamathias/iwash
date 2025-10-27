-- Rollback multi-vendor migration

-- Drop indexes
DROP INDEX IF EXISTS idx_vendor_payouts_created_at;
DROP INDEX IF EXISTS idx_vendor_payouts_status;
DROP INDEX IF EXISTS idx_vendor_payouts_vendor_id;
DROP INDEX IF EXISTS idx_notification_preferences_user_id;
DROP INDEX IF EXISTS idx_vendor_availability_vendor_id;
DROP INDEX IF EXISTS idx_vendor_service_areas_city;
DROP INDEX IF EXISTS idx_vendor_service_areas_vendor_id;
DROP INDEX IF EXISTS idx_reviews_rating;
DROP INDEX IF EXISTS idx_reviews_booking_id;
DROP INDEX IF EXISTS idx_reviews_user_id;
DROP INDEX IF EXISTS idx_reviews_vendor_id;
DROP INDEX IF EXISTS idx_services_featured;
DROP INDEX IF EXISTS idx_services_vendor_id;
DROP INDEX IF EXISTS idx_vendors_location;
DROP INDEX IF EXISTS idx_vendors_rating;
DROP INDEX IF EXISTS idx_vendors_is_active;
DROP INDEX IF EXISTS idx_vendors_city;
DROP INDEX IF EXISTS idx_vendors_user_id;
DROP INDEX IF EXISTS idx_users_is_active;
DROP INDEX IF EXISTS idx_users_role;

-- Drop tables
DROP TABLE IF EXISTS vendor_payouts;
DROP TABLE IF EXISTS notification_preferences;
DROP TABLE IF EXISTS vendor_availability;
DROP TABLE IF EXISTS vendor_service_areas;
DROP TABLE IF EXISTS reviews;
DROP TABLE IF EXISTS vendors;

-- Remove added columns from services
ALTER TABLE services DROP COLUMN IF EXISTS is_featured;
ALTER TABLE services DROP COLUMN IF EXISTS vendor_id;

-- Remove added columns from users
ALTER TABLE users DROP COLUMN IF EXISTS updated_at;
ALTER TABLE users DROP COLUMN IF EXISTS email_verified;
ALTER TABLE users DROP COLUMN IF EXISTS is_active;
ALTER TABLE users DROP COLUMN IF EXISTS role;

-- Drop user_role enum
DROP TYPE IF EXISTS user_role;
