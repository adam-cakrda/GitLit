pub mod mongo;
pub mod mysql;

use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Mongo error: {0}")]
    Mongo(#[from] mongodb::error::Error),
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Clone, Debug)]
pub enum Database {
    Mongo(mongo::Database),
    MySql(mysql::Database),
}

impl Database {
    pub async fn init() -> Self {
        let backend = env::var("DATABASE").unwrap_or_else(|_| "mongo".to_string());
        let auto_mysql = env::var("DATABASE_URL").ok().map(|u| u.starts_with("mysql://")).unwrap_or(false);
        if backend.eq_ignore_ascii_case("mysql") || auto_mysql {
            Database::MySql(mysql::Database::init().await)
        } else {
            Database::Mongo(mongo::Database::init().await)
        }
    }
}

impl Database {
    pub async fn create_user(&self, user: crate::models::User) -> Result<(), DbError> {
        match self {
            Database::Mongo(db) => {
                db.create_user(user).await?;
                Ok(())
            }
            Database::MySql(db) => {
                db.create_user(user).await?;
                Ok(())
            }
        }
    }

    pub async fn create_repository(&self, repository: crate::models::Repository) -> Result<(), DbError> {
        match self {
            Database::Mongo(db) => {
                db.create_repository(repository).await?;
                Ok(())
            }
            Database::MySql(db) => {
                db.create_repository(repository).await?;
                Ok(())
            }
        }
    }

    pub async fn find_user_by_login(&self, login: &str) -> Result<Option<crate::models::User>, DbError> {
        match self {
            Database::Mongo(db) => {
                db.find_user_by_login(login).await.map_err(Into::into)
            }
            Database::MySql(db) => {
                db.find_user_by_login(login).await.map_err(Into::into)
            }
        }
    }

    pub async fn find_user_by_id(&self, id: &str) -> Result<Option<crate::models::User>, DbError> {
        match self {
            Database::Mongo(db) => {
                db.find_user_by_id(id).await.map_err(Into::into)
            }
            Database::MySql(db) => {
                db.find_user_by_id(id).await.map_err(Into::into)
            }
        }
    }

    pub async fn create_token(&self, token: crate::models::Token) -> Result<(), DbError> {
        match self {
            Database::Mongo(db) => {
                db.create_token(token).await?;
                Ok(())
            }
            Database::MySql(db) => {
                db.create_token(token).await?;
                Ok(())
            }
        }
    }

    pub async fn find_repo(&self, id: String) -> Result<Option<crate::models::Repository>, DbError> {
        match self {
            Database::Mongo(db) => {
                db.find_repo(id).await.map_err(Into::into)
            }
            Database::MySql(db) => {
                db.find_repo(id).await.map_err(Into::into)
            }
        }
    }

    pub async fn is_repo_exists(&self, user_id: &String, name: &String) -> Result<bool, DbError> {
        match self {
            Database::Mongo(db) => {
                db.is_repo_exists(user_id, name).await.map_err(Into::into)
            }
            Database::MySql(db) => {
                db.is_repo_exists(user_id, name).await.map_err(Into::into)
            }
        }
    }

    pub async fn find_repo_by_user_and_name(&self, user_id: &str, name: &str) -> Result<Option<crate::models::Repository>, DbError> {
        match self {
            Database::Mongo(db) => {
                db.find_repo_by_user_and_name(user_id, name).await.map_err(Into::into)
            }
            Database::MySql(db) => {
                db.find_repo_by_user_and_name(user_id, name).await.map_err(Into::into)
            }
        }
    }

    pub async fn find_token(&self, token_value: &str) -> Result<Option<crate::models::Token>, DbError> {
        match self {
            Database::Mongo(db) => {
                db.find_token(token_value).await.map_err(Into::into)
            }
            Database::MySql(db) => {
                db.find_token(token_value).await.map_err(Into::into)
            }
        }
    }

    pub async fn delete_token(&self, token_value: &str) -> Result<u64, DbError> {
        match self {
            Database::Mongo(db) => {
                db.delete_token(token_value).await.map_err(Into::into)
            }
            Database::MySql(db) => {
                db.delete_token(token_value).await.map_err(Into::into)
            }
        }
    }

    pub async fn delete_repository_by_id(&self, id: &str) -> Result<u64, DbError> {
        match self {
            Database::Mongo(db) => {
                db.delete_repository_by_id(id).await.map_err(Into::into)
            }
            Database::MySql(db) => {
                db.delete_repository_by_id(id).await.map_err(Into::into)
            }
        }
    }

    pub async fn find_repos_with_filter_sort(&self, filter: bson::Document, sort: bson::Document) -> Result<Vec<crate::models::Repository>, DbError> {
        match self {
            Database::Mongo(db) => {
                db.find_repos_with_filter_sort(filter, sort).await.map_err(Into::into)
            }
            Database::MySql(db) => {
                db.find_repos_with_filter_sort(filter, sort).await.map_err(Into::into)
            }
        }
    }
}