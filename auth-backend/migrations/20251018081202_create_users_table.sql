-- Add migration script here

-- 启用 UUID 支持
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 创建 users 表
CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

  username TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL
)