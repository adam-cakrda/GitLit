mod exporters;
mod helpers;

use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use std::env;
use tracing::info;

#[derive(Clone, Debug)]
pub struct Database {
    pub pool: Pool<MySql>,
}

impl Database {
    pub async fn init() -> Self {
        let user = env::var("DATABASE_USER").unwrap_or_else(|_| "root".to_string());
        let pass = env::var("DATABASE_PASSWORD").unwrap_or_else(|_| "password".to_string());
        let host = env::var("DATABASE_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("DATABASE_PORT").unwrap_or_else(|_| "3306".to_string());
        let name = env::var("DATABASE_NAME").unwrap_or_else(|_| "gitlit".to_string());
        let url = format!("mysql://{}:{}@{}:{}/{}", user, pass, host, port, name);

        info!("Connecting to MySQL database");
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .expect("Failed to connect to MySQL");
        
        let _ = sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS users (
                id VARCHAR(32) PRIMARY KEY,
                username VARCHAR(255) NOT NULL UNIQUE,
                email VARCHAR(255) NOT NULL UNIQUE,
                password VARCHAR(255) NOT NULL,
                display_name VARCHAR(255) NOT NULL,
                avatar_url TEXT NULL,
                created_at DATETIME NOT NULL
            )
        "#)
        .execute(&pool)
        .await;

        let _ = sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS repositories (
                id VARCHAR(32) PRIMARY KEY,
                user_id VARCHAR(32) NOT NULL,
                name VARCHAR(255) NOT NULL,
                description TEXT NOT NULL,
                is_private BOOLEAN NOT NULL,
                forked_from VARCHAR(32) NULL,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                INDEX idx_user (user_id),
                INDEX idx_priv (is_private)
            )
        "#)
        .execute(&pool)
        .await;

        let _ = sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS tokens (
                id VARCHAR(36) PRIMARY KEY,
                user_id VARCHAR(32) NOT NULL,
                token VARCHAR(64) NOT NULL UNIQUE,
                created_at DATETIME NOT NULL,
                expires_at DATETIME NULL,
                INDEX idx_token (token)
            )
        "#)
        .execute(&pool)
        .await;

        Database { pool }
    }
}
