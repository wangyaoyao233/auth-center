use std::env;
use std::sync::Arc;

use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;
use sqlx::PgPool;

use crate::repositories::{PostgresRepository, UserRepository};

mod handlers;
mod models;
mod repositories;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create database pool.");

    // 将 Pool 放入 Arc 中以便安全共享
    let pool_arc = Arc::new(pool);

    // 创建具体的 Repository
    let repo = PostgresRepository::new(pool_arc);

    // 将具体的 repo 向上转型 (cast) 为 抽象的 Trait Object
    //    `Arc<dyn UserRepository>` 是在 Actix 中注入 Trait 的标准方式
    let repo_data: Arc<dyn UserRepository> = Arc::new(repo);

    println!("🚀 服务器启动于 http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(repo_data.clone()))
            .configure(handlers::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
