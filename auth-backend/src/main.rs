use std::env;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, http, web};
use dotenvy::dotenv;
use sqlx::PgPool;

use crate::repositories::{PostgresRepository, UserRepository};

mod handlers;
mod models;
mod repositories;
mod utils;

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
        // 定义 CORS 策略
        let _cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            // 如果有其他环境，也添加它们
            // .allowed_origin("https-your-production-frontend.com")
            // 允许的 HTTP 方法
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            // 允许前端发送的头部
            // 这对于 'Content-Type: application/json' 和 JWT 的 'Authorization'至关重要
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            // 允许浏览器发送 cookies 和 Authorization 头部
            .supports_credentials()
            // 缓存 "preflight" (OPTIONS) 请求的结果 1 小时
            .max_age(3600);

        App::new()
            .wrap(_cors)
            .app_data(web::Data::from(repo_data.clone()))
            .service(web::scope("/api").configure(handlers::config))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
