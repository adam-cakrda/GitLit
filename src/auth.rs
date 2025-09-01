use crate::db::Database;
use crate::errors::AuthError;
use crate::models::{User, Token};
use mongodb::bson::{oid::ObjectId, DateTime};
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};

const TOKEN_TTL_SECS: i64 = 24 * 60 * 60; // 24 hours

fn now_millis() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
}

fn dt_from_millis(ms: i64) -> DateTime {
    DateTime::from_millis(ms)
}

pub async fn register(db: &Database, username: String, email: String, password: String) -> Result<(), AuthError> {
    let pw_len = password.chars().count();
    if pw_len < 8 || pw_len > 128 {
        return Err(AuthError::InvalidCredentials);
    }

    match db.find_user_by_login(&username).await {
        Ok(Some(_)) => return Err(AuthError::InvalidCredentials),
        Ok(None) => {},
        Err(e) => return Err(AuthError::Internal(e.to_string())),
    }
    match db.find_user_by_login(&email).await {
        Ok(Some(_)) => return Err(AuthError::InvalidCredentials),
        Ok(None) => {},
        Err(e) => return Err(AuthError::Internal(e.to_string())),
    }

    let password_hash = hash(password, DEFAULT_COST)
        .map_err(|e| AuthError::Internal(e.to_string()))?;

    let now = now_millis();

    let user = User {
        _id: ObjectId::new(),
        username: username.clone(),
        email: email.clone(),
        password: password_hash,
        display_name: username.clone(),
        avatar_url: None,
        created_at: dt_from_millis(now),
    };

    db.create_user(user)
        .await
        .map_err(|e| AuthError::Internal(e.to_string()))?;

    Ok(())
}

pub async fn login(db: &Database, login: String, password: String) -> Result<String, AuthError> {
    let user = db
        .find_user_by_login(&login)
        .await
        .map_err(|e| AuthError::Internal(e.to_string()))?
        .ok_or(AuthError::InvalidCredentials)?;

    let ok = verify(password, &user.password)
        .map_err(|e| AuthError::Internal(e.to_string()))?;
    if !ok {
        return Err(AuthError::InvalidCredentials);
    }

    let now = now_millis();
    let exp = now + TOKEN_TTL_SECS * 1000;

    let token_value = Uuid::new_v4().to_string();
    let token = Token {
        _id: ObjectId::new(),
        user: user._id,
        token: token_value.clone(),
        created_at: dt_from_millis(now),
        expires_at: Some(dt_from_millis(exp)),
    };

    db.create_token(token)
        .await
        .map_err(|e| AuthError::Internal(e.to_string()))?;

    Ok(token_value)
}

pub async fn logout(db: &Database, token: String) -> Result<(), AuthError> {
    let deleted = db
        .delete_token(&token)
        .await
        .map_err(|e| AuthError::Internal(e.to_string()))?;

    if deleted == 0 {
        return Err(AuthError::InvalidCredentials);
    }

    Ok(())
}

pub async fn auth(db: &Database, token: String) -> Result<ObjectId, AuthError> {
    let t = db
        .find_token(&token)
        .await
        .map_err(|e| AuthError::Internal(e.to_string()))?
        .ok_or(AuthError::InvalidCredentials)?;

    if let Some(exp) = t.expires_at {
        let now = now_millis();
        if dt_from_millis(now) > exp {
            let _ = db.delete_token(&token).await;
            return Err(AuthError::InvalidCredentials);
        }
    }

    Ok(t.user)
}