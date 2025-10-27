# 🧺 iWash - A Highly Performant Multi-Vendor Laundry Booking Platform API.

A production-ready, scalable multi-vendor laundry booking platform built with Rust, designed to connect customers with local laundry service providers.

## 🌟 Overview

iWash is a modern **multi-vendor marketplace** that enables:
- **Customers**: Browse vendors, book laundry services, track orders, and rate services
- **Vendors**: Register their laundry business, manage services, handle bookings, and track earnings
- **Admins**: Oversee platform operations, verify vendors, and manage the ecosystem

For full documentation, see the complete README in the repository.

**Version**: 0.1.0  
**Status**: Active Development 🚀  
**Last Updated**: October 27, 2025

---

## 🗄️ Database Migrations

iWash uses **sqlx-cli** for version-controlled database migrations. This ensures safe, trackable schema changes across development, staging, and production environments.

### Prerequisites

Install sqlx-cli if you haven't already:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

Verify installation:

```bash
sqlx --version  # Should show: sqlx-cli 0.8.6
```

### Running Migrations

**1. Local Development:**

```bash
# Run all pending migrations on your local database
sqlx migrate run --database-url $DATABASE_URL

# For test database
sqlx migrate run --database-url $TEST_DATABASE_URL
```

**2. Check Migration Status:**

```bash
# See which migrations have been applied
sqlx migrate info --database-url $DATABASE_URL
```

**3. Rollback Migrations:**

```bash
# Rollback the last migration
sqlx migrate revert --database-url $DATABASE_URL

# Rollback multiple migrations
sqlx migrate revert --database-url $DATABASE_URL
sqlx migrate revert --database-url $DATABASE_URL
```

### Migration Files

Migrations are located in the `migrations/` folder with timestamp-based naming:

```
migrations/
├── 20251027212411_initial_schema.up.sql        # Users table
├── 20251027212411_initial_schema.down.sql      # Rollback for users
├── 20251027212426_bookings_schema.up.sql       # Services, bookings, booking_items
├── 20251027212426_bookings_schema.down.sql     # Rollback for bookings
├── 20251027212437_multi_vendor.up.sql          # Multi-vendor architecture
└── 20251027212437_multi_vendor.down.sql        # Rollback for multi-vendor
```

### Creating New Migrations

When you need to add new schema changes:

```bash
# Create a new migration with up and down files
sqlx migrate add -r your_migration_name

# Example: Adding a payments table
sqlx migrate add -r add_payments_table
```

This creates two files:
- `YYYYMMDDHHMMSS_your_migration_name.up.sql` - Schema changes
- `YYYYMMDDHHMMSS_your_migration_name.down.sql` - Rollback SQL

### Cloud Deployment (AWS RDS, Google Cloud SQL, Supabase, etc.)

For production deployments to cloud databases:

**1. Set up SSL connection (required for most cloud providers):**

```bash
# .env.production
DATABASE_URL=postgres://username:password@your-cloud-host.com:5432/iwash_production?sslmode=require
```

**2. Run migrations during deployment:**

```bash
# In your CI/CD pipeline (GitHub Actions, GitLab CI, etc.)
sqlx migrate run --database-url $DATABASE_URL
```

**3. Example GitHub Actions workflow:**

```yaml
- name: Run Database Migrations
  env:
    DATABASE_URL: ${{ secrets.DATABASE_URL }}
  run: |
    cargo install sqlx-cli --no-default-features --features postgres
    sqlx migrate run
```

### Migration Tracking

sqlx-cli automatically creates a `_sqlx_migrations` table in your database to track:
- Which migrations have been applied
- When they were applied
- Migration checksums for integrity verification

View migration history:

```bash
psql $DATABASE_URL -c "SELECT version, description, installed_on FROM _sqlx_migrations ORDER BY installed_on;"
```

### Best Practices

✅ **DO:**
- Always create both `.up.sql` and `.down.sql` files for reversibility
- Test migrations on a copy of production data before deploying
- Use transactions for complex multi-step migrations
- Keep migrations small and focused
- Review migration checksums in `_sqlx_migrations` for integrity

❌ **DON'T:**
- Modify existing migration files after they've been applied to production
- Delete migration files from the `migrations/` folder
- Skip testing rollback scripts
- Make breaking schema changes without a migration plan

### Troubleshooting

**Migration checksum mismatch:**
```bash
# This means a migration file was modified after being applied
# Solution: Revert the migration, fix it, and re-apply
sqlx migrate revert --database-url $DATABASE_URL
# Edit the migration file
sqlx migrate run --database-url $DATABASE_URL
```

**Connection errors to cloud database:**
```bash
# Ensure SSL mode is set for cloud databases
DATABASE_URL=postgres://user:pass@host/db?sslmode=require
```

**Need to reset database completely:**
```bash
# Drop and recreate (ONLY for development!)
psql "postgres://postgres:password@localhost/postgres" -c "DROP DATABASE iwash_db;"
psql "postgres://postgres:password@localhost/postgres" -c "CREATE DATABASE iwash_db;"
sqlx migrate run --database-url $DATABASE_URL
```

---
