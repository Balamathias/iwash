-- Rollback bookings schema

DROP INDEX IF EXISTS idx_services_is_active;
DROP INDEX IF EXISTS idx_booking_items_booking_id;
DROP INDEX IF EXISTS idx_bookings_scheduled_pickup;
DROP INDEX IF EXISTS idx_bookings_status;
DROP INDEX IF EXISTS idx_bookings_user_id;

DROP TABLE IF EXISTS booking_items;
DROP TABLE IF EXISTS bookings;
DROP TYPE IF EXISTS booking_status;
DROP TABLE IF EXISTS services;
