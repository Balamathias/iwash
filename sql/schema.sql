-- iWash initial schema

CREATE TABLE IF NOT EXISTS users (
  id uuid PRIMARY KEY,
  email text NOT NULL UNIQUE,
  password_hash text NOT NULL,
  full_name text NULL,
  phone text NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);
