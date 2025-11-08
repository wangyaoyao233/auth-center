use crate::models::{RegisterRequest, User};
use anyhow::{Ok, Result};
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

// --- 1. "契约" / Trait ---
// 这个 trait 定义了我们的业务逻辑需要哪些数据库操作
// 它必须是 Send + Sync，因为 web::Data 会在多线程间共享它
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_user_by_username(&self, username: &str) -> Result<User>;
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User>;
    async fn create_user(&self, req: &RegisterRequest, password_hash: &str) -> Result<User>;
    async fn update_user_otp(
        &self,
        user_id: &Uuid,
        otp_base32: &str,
        otp_auth_url: &str,
    ) -> Result<()>;
    async fn disable_user_otp(&self, user_id: &Uuid) -> Result<()>;
    async fn verify_user_otp(&self, user_id: &Uuid) -> Result<()>;
}

// --- 2. "PostgreSQL 实现" ---
// 这是一个实现了 UserRepository trait 的具体结构体
pub struct PostgresRepository {
    pool: Arc<PgPool>, // 持有数据库连接池
}

impl PostgresRepository {
    // 构造函数
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

// 为 PostgresRepository 实现 UserRepository "契约"
#[async_trait]
impl UserRepository for PostgresRepository {
    async fn get_user_by_username(&self, username: &str) -> Result<User> {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", username)
            .fetch_one(self.pool.as_ref()) // 从池中获取连接并执行
            .await?; // 从池中获取连接并执行

        // Ok(user) 会被 anyhow::Result 捕获
        Ok(user)
    }

    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User> {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
            .fetch_one(self.pool.as_ref())
            .await?;
        Ok(user)
    }

    async fn create_user(&self, req: &RegisterRequest, password_hash: &str) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *",
            req.username,
            req.email,
            password_hash
        )
        .fetch_one(self.pool.as_ref())
        .await?;
        Ok(user)
    }

    async fn update_user_otp(
        &self,
        user_id: &Uuid,
        otp_base32: &str,
        otp_auth_url: &str,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET otp_enabled = true, otp_base32 = $1, otp_auth_url = $2 WHERE id = $3",
            otp_base32,
            otp_auth_url,
            user_id
        )
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn verify_user_otp(&self, user_id: &Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET otp_verified = true WHERE id = $1",
            user_id
        )
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    async fn disable_user_otp(&self, user_id: &Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE users SET otp_enabled = false, otp_verified = false, otp_base32 = NULL, otp_auth_url = NULL WHERE id = $1",
            user_id
        )
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }
}

// --- 3. （未来）"DynamoDB 实现" ---
/*
use aws_sdk_dynamodb::Client as DynamoDbClient;

pub struct DynamoDbRepository {
    client: Arc<DynamoDbClient>,
}

impl DynamoDbRepository {
    pub fn new(client: Arc<DynamoDbClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UserRepository for DynamoDbRepository {
    async fn get_user_by_username(&self, username: &str) -> Result<User> {
        // ... 在这里写你的 DynamoDB get_item 逻辑 ...
        // ... 将 DynamoDB 的输出映射到 User 结构体 ...
        // ... 如果找不到，返回 Err(anyhow::anyhow!("User not found")) ...
        todo!() // 尚未实现
    }
}
*/
