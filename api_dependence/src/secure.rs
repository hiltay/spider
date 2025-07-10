use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Json, RequestPartsExt,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use chrono::{Local, TimeDelta};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::distr::{Alphanumeric, SampleString};

use serde::{Deserialize, Serialize};
use serde_json::json;

pub fn generate_secret_key() -> String {
    Alphanumeric.sample_string(&mut rand::rng(), 32)
}

pub fn generate_new_token(password: &str) -> Result<String, AuthError> {
    let secret = generate_secret_key();
    // println!("{}", secret);
    let secret_key = Keys::new(secret.as_bytes());
    let now = Local::now();
    let expa = now + TimeDelta::days(7);
    let claims = Claims {
        // pwd hash
        password_hash: create_password_hash(password),
        // Mandatory expiry time as UTC timestamp
        exp: expa.timestamp() as usize, // current day + 7 days
    };
    // Create the authorization token
    let token = encode(&Header::default(), &claims, &secret_key.encoding)
        .map_err(|_| AuthError::TokenCreation)?;
    Ok(token)
}

pub fn create_password_hash(plain_pwd: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2
        .hash_password(plain_pwd.as_bytes(), &salt)
        .unwrap()
        .to_string();

    return password_hash;
}

pub fn verify_password(plain_pwd: &str, hash_pwd: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash_pwd).unwrap();
    Argon2::default()
        .verify_password(plain_pwd.as_bytes(), &parsed_hash)
        .is_ok()
}

impl AuthBody {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        // Decode the user data
        let token_data = decode::<Claims>(
            bearer.token(),
            &Keys::new("123".as_bytes()).decoding,
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub password_hash: String,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
pub struct AuthBody {
    access_token: String,
    token_type: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    client_id: String,
    client_secret: String,
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}
