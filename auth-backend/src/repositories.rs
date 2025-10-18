use crate::models::User;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

// --- 1. "契约" / Trait ---
// 这个 trait 定义了我们的业务逻辑需要哪些数据库操作
// 它必须是 Send + Sync，因为 web::Data 会在多线程间共享它
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_user_by_username(&self, username: &str) -> Result<User>;
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
        let user = sqlx::query_as!(
            User,
            "SELECT id, username, password_hash FROM users WHERE username = $1",
            username
        )
        .fetch_one(self.pool.as_ref()) // 从池中获取连接并执行
        .await?; // 从池中获取连接并执行

        // Ok(user) 会被 anyhow::Result 捕获
        Ok(user)
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
