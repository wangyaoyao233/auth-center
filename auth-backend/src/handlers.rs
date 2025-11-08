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
        ApiResponse, Claims, DisableOTPSchema, GenerateOTPSchema, LoginMfaData, LoginRequest,
        RegisterRequest, VerifyOTPSchema,
    },
    repositories::UserRepository,
};

fn get_jwt_secret() -> String {
    env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}

#[post("/auth/login")]
async fn login(
    data: web::Json<LoginRequest>,
    repo: web::Data<dyn UserRepository>,
) -> impl Responder {
    let user = match repo.get_user_by_email(&data.email).await {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                status: "error".to_string(),
                message: "Invalid credentials".to_string(),
                data: None,
            });
        }
    };

    let is_valid = match verify(&data.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                status: "error".to_string(),
                message: "Password verification failed".to_string(),
                data: None,
            });
        }
    };

    if !is_valid {
        return HttpResponse::Unauthorized().json(ApiResponse::<()> {
            status: "error".to_string(),
            message: "Invalid credentials".to_string(),
            data: None,
        });
    }

    let exp = (Utc::now() + Duration::hours(1)).timestamp() as usize;
    let claims = Claims {
        sub: user.username.clone(),
        exp,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_jwt_secret().as_bytes()),
    ) {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                status: "error".to_string(),
                message: "Could not create token".to_string(),
                data: None,
            });
        }
    };

    HttpResponse::Ok().json(ApiResponse {
        status: "success".to_string(),
        message: "Login successful".to_string(),
        data: Some(LoginMfaData { mfa_token: token }),
    })
}

#[post("/auth/register")]
async fn register(
    data: web::Json<RegisterRequest>,
    repo: web::Data<dyn UserRepository>,
) -> impl Responder {
    let password_hash = match hash(&data.password, bcrypt::DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                status: "error".to_string(),
                message: "Could not hash password".to_string(),
                data: None,
            });
        }
    };

    match repo.create_user(&data, &password_hash).await {
        Ok(user) => HttpResponse::Ok().json(ApiResponse {
            status: "success".to_string(),
            message: "User registered successfully".to_string(),
            data: Some(user),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            status: "error".to_string(),
            message: e.to_string(),
            data: None,
        }),
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
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiResponse::<()> {
                status: "error".to_string(),
                message: "Invalid user_id format".to_string(),
                data: None,
            });
        }
    };

    if (repo.get_user_by_id(&user_id).await).is_err() {
        return HttpResponse::NotFound().json(ApiResponse::<()> {
            status: "error".to_string(),
            message: "User not found".to_string(),
            data: None,
        });
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
        Ok(_) => HttpResponse::Ok().json(ApiResponse {
            status: "success".to_string(),
            message: "OTP generated successfully".to_string(),
            data: Some(json!({
                "otp_base32": otp_base32,
                "otp_auth_url": otp_auth_url
            })),
        }),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            status: "error".to_string(),
            message: "Failed to update user OTP info".to_string(),
            data: None,
        }),
    }
}

#[post("/auth/otp/verify")]
async fn verify_otp(
    data: web::Json<VerifyOTPSchema>,
    repo: web::Data<dyn UserRepository>,
) -> impl Responder {
    let user_id = match Uuid::from_str(&data.user_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiResponse::<()> {
                status: "error".to_string(),
                message: "Invalid user_id format".to_string(),
                data: None,
            });
        }
    };

    let user = match repo.get_user_by_id(&user_id).await {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::NotFound().json(ApiResponse::<()> {
                status: "error".to_string(),
                message: "User not found".to_string(),
                data: None,
            });
        }
    };

    let otp_base32 = match &user.otp_base32 {
        Some(base32) => base32.clone(),
        None => {
            return HttpResponse::BadRequest().json(ApiResponse::<()> {
                status: "error".to_string(),
                message: "OTP not set up for this user".to_string(),
                data: None,
            });
        }
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
        if repo.verify_user_otp(&user_id).await.is_err() {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                status: "error".to_string(),
                message: "Failed to update OTP status".to_string(),
                data: None,
            });
        };

        HttpResponse::Ok().json(ApiResponse::<()> {
            status: "success".to_string(),
            message: "OTP verified successfully".to_string(),
            data: None,
        })
    } else {
        HttpResponse::Unauthorized().json(ApiResponse::<()> {
            status: "error".to_string(),
            message: "Invalid OTP token".to_string(),
            data: None,
        })
    }
}

#[post("/auth/otp/validate")]
async fn validate_otp(
    data: web::Json<VerifyOTPSchema>,
    repo: web::Data<dyn UserRepository>,
) -> impl Responder {
    let user_id = match Uuid::from_str(&data.user_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiResponse::<()> {
                status: "error".to_string(),
                message: "Invalid user_id format".to_string(),
                data: None,
            });
        }
    };

    let user = match repo.get_user_by_id(&user_id).await {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::NotFound().json(ApiResponse::<()> {
                status: "error".to_string(),
                message: "User not found".to_string(),
                data: None,
            });
        }
    };

    if !user.otp_enabled.unwrap_or(false) {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            status: "error".to_string(),
            message: "OTP is not enabled for this user".to_string(),
            data: None,
        });
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
        return HttpResponse::Unauthorized().json(ApiResponse::<()> {
            status: "error".to_string(),
            message: "Invalid OTP token".to_string(),
            data: None,
        });
    }

    HttpResponse::Ok().json(ApiResponse::<()> {
        status: "success".to_string(),
        message: "OTP validated successfully".to_string(),
        data: None,
    })
}

#[post("/auth/otp/disable")]
async fn disable_otp(
    data: web::Json<DisableOTPSchema>,
    repo: web::Data<dyn UserRepository>,
) -> impl Responder {
    let user_id = match Uuid::from_str(&data.user_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiResponse::<()> {
                status: "error".to_string(),
                message: "Invalid user_id format".to_string(),
                data: None,
            });
        }
    };

    let user = match repo.get_user_by_id(&user_id).await {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::NotFound().json(ApiResponse::<()> {
                status: "error".to_string(),
                message: "User not found".to_string(),
                data: None,
            });
        }
    };

    if !user.otp_enabled.unwrap_or(false) {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            status: "error".to_string(),
            message: "OTP is already disabled for this user".to_string(),
            data: None,
        });
    }

    if repo.update_user_otp(&user_id, "", "").await.is_err() {
        return HttpResponse::InternalServerError().json(ApiResponse::<()> {
            status: "error".to_string(),
            message: "Failed to disable OTP".to_string(),
            data: None,
        });
    }

    if repo.disable_user_otp(&user_id).await.is_err() {
        return HttpResponse::InternalServerError().json(ApiResponse::<()> {
            status: "error".to_string(),
            message: "Failed to disable OTP".to_string(),
            data: None,
        });
    }

    HttpResponse::Ok().json(ApiResponse::<()> {
        status: "success".to_string(),
        message: "OTP disabled successfully".to_string(),
        data: None,
    })
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
