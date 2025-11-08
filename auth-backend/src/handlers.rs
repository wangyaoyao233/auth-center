use actix_web::{HttpResponse, Responder, get, post, web};
use bcrypt::{hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use rand::Rng;
use serde_json::json;
use std::env;
use std::str::FromStr;
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;

use crate::{
    models::{
        Claims, DisableOTPSchema, GenerateOTPSchema, LoginRequest, LoginResponse, RegisterRequest,
        VerifyOTPSchema,
    },
    // 导入 抽象的 Trait，而不是 PgPool！
    repositories::UserRepository,
};

fn get_jwt_secret() -> String {
    env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}

#[post("/auth/login")]
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

#[post("/auth/register")]
async fn register(
    data: web::Json<RegisterRequest>,
    repo: web::Data<dyn UserRepository>,
) -> impl Responder {
    let password_hash = match hash(&data.password, bcrypt::DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().json("Could not hash password"),
    };

    match repo.create_user(&data, &password_hash).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/auth/otp/generate")]
async fn generate_otp(
    data: web::Json<GenerateOTPSchema>,
    repo: web::Data<dyn UserRepository>,
) -> impl Responder {
    let user_id = match Uuid::from_str(&data.user_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json("Invalid user_id format"),
    };

    if (repo.get_user_by_id(&user_id).await).is_err() {
        return HttpResponse::NotFound().json("User not found");
    }

    let mut rng = rand::thread_rng();
    let data_byte: [u8; 21] = rng.r#gen();
    let base32_string = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &data_byte);

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Encoded(base32_string).to_bytes().unwrap(),
    )
    .unwrap();

    let otp_base32 = totp.get_secret_base32();
    let email = data.email.to_owned();
    let issuer = "AuthApp";
    let otp_auth_url =
        format!("otpauth://totp/{issuer}:{email}?secret={otp_base32}&issuer={issuer}");

    match repo
        .update_user_otp(&user_id, &otp_base32, &otp_auth_url)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({
            "otp_base32": otp_base32,
            "otp_auth_url": otp_auth_url
        })),
        Err(_) => HttpResponse::InternalServerError().json("Failed to update user OTP info"),
    }
}

#[post("/auth/otp/verify")]
async fn verify_otp(
    data: web::Json<VerifyOTPSchema>,
    repo: web::Data<dyn UserRepository>,
) -> impl Responder {
    let user_id = match Uuid::from_str(&data.user_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json("Invalid user_id format"),
    };

    let user = match repo.get_user_by_id(&user_id).await {
        Ok(user) => user,
        Err(_) => return HttpResponse::NotFound().json("User not found"),
    };

    let otp_base32 = match &user.otp_base32 {
        Some(base32) => base32.clone(),
        None => return HttpResponse::BadRequest().json("OTP not set up for this user"),
    };

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Encoded(otp_base32).to_bytes().unwrap(),
    )
    .unwrap();

    let is_valid = totp.check_current(&data.token).unwrap();

    if is_valid {
        match repo.verify_user_otp(&user_id).await {
            Ok(_) => (),
            Err(_) => {
                return HttpResponse::InternalServerError().json("Failed to update OTP status");
            }
        };

        HttpResponse::Ok().json("OTP verified successfully")
    } else {
        HttpResponse::Unauthorized().json("Invalid OTP token")
    }
}

#[post("/auth/otp/validate")]
async fn validate_otp(
    data: web::Json<VerifyOTPSchema>,
    repo: web::Data<dyn UserRepository>,
) -> impl Responder {
    let user_id = match Uuid::from_str(&data.user_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json("Invalid user_id format"),
    };

    let user = match repo.get_user_by_id(&user_id).await {
        Ok(user) => user,
        Err(_) => return HttpResponse::NotFound().json("User not found"),
    };

    if !user.otp_enabled.unwrap_or(false) {
        return HttpResponse::BadRequest().json("OTP is not enabled for this user");
    }

    let otp_base32 = user.otp_base32.to_owned().unwrap();

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Encoded(otp_base32).to_bytes().unwrap(),
    )
    .unwrap();

    let is_valid = totp.check_current(&data.token).unwrap();

    if !is_valid {
        return HttpResponse::Unauthorized().json("Invalid OTP token");
    }

    HttpResponse::Ok().json("OTP validated successfully")
}

#[post("/auth/otp/disable")]
async fn disable_otp(
    data: web::Json<DisableOTPSchema>,
    repo: web::Data<dyn UserRepository>,
) -> impl Responder {
    let user_id = match Uuid::from_str(&data.user_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json("Invalid user_id format"),
    };

    let user = match repo.get_user_by_id(&user_id).await {
        Ok(user) => user,
        Err(_) => return HttpResponse::NotFound().json("User not found"),
    };

    if !user.otp_enabled.unwrap_or(false) {
        return HttpResponse::BadRequest().json("OTP is already disabled for this user");
    }

    match repo
        .update_user_otp(&user_id, "", "") // 清空 OTP 信息
        .await
    {
        Ok(_) => (),
        Err(_) => return HttpResponse::InternalServerError().json("Failed to disable OTP"),
    }

    match repo.disable_user_otp(&user_id).await {
        Ok(_) => (),
        Err(_) => return HttpResponse::InternalServerError().json("Failed to disable OTP"),
    }
    HttpResponse::Ok().json("OTP disabled successfully")
}
// --- 关键的 Actix-web 模式 ---
// 这个函数会配置这个模块的所有路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(hello)
        .service(login)
        .service(register)
        .service(generate_otp)
        .service(verify_otp)
        .service(validate_otp)
        .service(disable_otp);
}
