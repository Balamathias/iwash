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
iwash/
 ├── src/
 │   ├── main.rs          # Entry point, sets up Axum router and DB
 │   ├── db.rs            # Database connection using SQLx + dotenvy
 │   ├── models.rs        # User, Booking, etc. structs & SQLx mappings
 │   ├── auth.rs          # Registration, login, JWT generation
 │   ├── middleware.rs    # AuthUser extractor for JWT validation
 │   ├── routes.rs        # Route definitions and handlers
 │   └── errors.rs        # Custom error handling with thiserror
 ├── .env                 # Contains DATABASE_URL
 ├── Cargo.toml           # Dependencies & build configuration
 └── README.md

⚙️ Key Dependencies
[dependencies]
I have included the key dependencies in Cargo.toml:
You could add more as needed.

🧩 Current Stage

We’re currently setting up the core backend foundation:

✅ Rust project initialized with cargo new iwash

✅ Dependencies added (Axum, SQLx, dotenvy, JWT, bcrypt, etc.)


Next steps:

Database connection (db.rs) implemented

Health check route working

Implement the User model

Add Register/Login routes

Secure endpoints using JWT middleware

Build the Bookings module

🧠 Development Notes for the AI Agent

When assisting in this project, the AI agent should:

Maintain idiomatic Rust patterns (ownership, lifetimes, error handling)

Use async/await correctly with tokio

Keep code modular (separate auth, models, db, etc.)

Follow RESTful API best practices

Output complete, compilable snippets

Optimize for clarity and maintainability (commented examples)

Assume PostgreSQL is used locally via .env

Example .env
DATABASE_URL=postgres://postgres:password@localhost/iwash_db
JWT_SECRET=super_secret_key

🧭 Goal

To produce a production-ready Rust backend that can be easily connected to a React Native frontend for a laundry service platform (user bookings, payments, order tracking).

MAKE SURE WE DEVELOP, BUILD AND RUN ONE STEP AT A TIME! We have to ensure everything is working properly before moving on to the next step.

Also set up Robust professional logging using the tracing crate to log important events, errors, and request details.