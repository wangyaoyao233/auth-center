use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use std::env;
use uuid::Uuid;

use crate::models::Claims;

fn get_jwt_secret() -> String {
    env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}

pub fn generate_mfa_token(user_id: &Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = (Utc::now() + Duration::minutes(5)).timestamp() as usize;
    // "amr": ["pwd"]
    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        aud: Some("mfa-verification".to_string()),
        amr: Some(vec!["pwd".to_string()]),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_jwt_secret().as_bytes()),
    )
}

pub fn generate_access_token(user_id: &Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    // "amr": ["pwd", "mfa"]
    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        aud: Some("urn:auth-center:api".to_string()),
        amr: Some(vec!["pwd".to_string(), "mfa".to_string()]),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_jwt_secret().as_bytes()),
    )
}

pub fn generate_refresh_token(user_id: &Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = (Utc::now() + Duration::days(7)).timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        aud: Some("refresh-token".to_string()),
        amr: None,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_jwt_secret().as_bytes()),
    )
}

pub fn validate_mfa_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.set_audience(&["mfa-verification"]);
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(get_jwt_secret().as_bytes()),
        &validation,
    )?;
    if let Some(amr) = &decoded.claims.amr {
        // 它必须由 "pwd" 生成，且尚未通过 "mfa"
        if amr.contains(&"pwd".to_string()) && !amr.contains(&"mfa".to_string()) {
            Ok(decoded.claims)
        } else {
            // 这是一个无效的MFA令牌（例如，它可能是一个已完成MFA的令牌）
            Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ))
        }
    } else {
        // 缺少 AMR 声明，不是一个有效的MFA令牌
        Err(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidToken,
        ))
    }
}

pub fn validate_access_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.set_audience(&["urn:auth-center:api"]);
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(get_jwt_secret().as_bytes()),
        &validation,
    )?;

    if let Some(amr) = &decoded.claims.amr {
        if !amr.contains(&"mfa".to_string()) {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ));
        }
    } else {
        return Err(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidToken,
        ));
    }

    Ok(decoded.claims)
}
