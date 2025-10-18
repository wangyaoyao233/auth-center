use actix_web::{HttpResponse, Responder, get, post, web};
use bcrypt::verify;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use std::env;

use crate::{
    models::{Claims, LoginRequest, LoginResponse},
    // 导入 抽象的 Trait，而不是 PgPool！
    repositories::UserRepository,
};

fn get_jwt_secret() -> String {
    env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}

#[post("/login")]
async fn login(
    data: web::Json<LoginRequest>,
    // 注入抽象的 Repository Trait
    repo: web::Data<dyn UserRepository>,
) -> impl Responder {
    // 调用 "契约" 中定义的方法
    let user = match repo.get_user_by_username(&data.username).await {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::Unauthorized().json("Invalid credentials");
        }
    };

    let is_valid = match verify(&data.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(_) => {
            return HttpResponse::InternalServerError().json("Password verification failed");
        }
    };

    if !is_valid {
        return HttpResponse::Unauthorized().json("Invalid credentials");
    }

    let exp = (Utc::now() + Duration::hours(1)).timestamp() as usize;
    let claims = Claims {
        sub: data.username.clone(),
        exp,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_jwt_secret().as_bytes()),
    ) {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().json("Could not create token"),
    };

    HttpResponse::Ok().json(LoginResponse { token })
}

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/hash")]
async fn generate_hash() -> impl Responder {
    // 使用 web::block 将阻塞工作移出 async 线程
    //    web::block 会在另一个线程上运行这个闭包
    let hash_result = web::block(|| bcrypt::hash("password123", bcrypt::DEFAULT_COST)).await; // .await 等待那个线程完成

    // web::block 返回一个 Result，你需要处理它
    match hash_result {
        // web::block 成功了，并且 bcrypt::hash() 也成功了
        Ok(Ok(hash)) => HttpResponse::Ok().body(hash),
        // web::block 成功了，但 bcrypt::hash() 失败了
        Ok(Err(e)) => HttpResponse::InternalServerError().body(format!("Bcrypt error: {}", e)),
        // web::block 本身失败了 (比如线程池满了)
        Err(e) => HttpResponse::InternalServerError().body(format!("Blocking pool error: {}", e)),
    }
}

// --- 关键的 Actix-web 模式 ---
// 这个函数会配置这个模块的所有路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(hello).service(login).service(generate_hash);
}
