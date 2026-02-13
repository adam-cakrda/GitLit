mod helpers;
mod models;

pub use models::*;

use mongodb::{Client, Collection};
use std::env;
use tracing::info;

#[derive(Clone, Debug)]
pub struct Database {
    users: Collection<User>,
    repositories: Collection<Repository>,
    tokens: Collection<Token>,
}

impl Database {
    pub async fn init() -> Self {
        let db_name = env::var("DATABASE_NAME")
            .unwrap_or_else(|_| "gitlit".to_string());

        let host = env::var("DATABASE_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("DATABASE_PORT").unwrap_or_else(|_| "27017".to_string());
        let user_opt = env::var("DATABASE_USER").ok();
        let pass_opt = env::var("DATABASE_PASSWORD").ok();
        let uri = match user_opt {
            Some(user) => {
                let pass = pass_opt.unwrap_or_default();
                format!("mongodb://{}:{}@{}:{}/{}?directConnection=true", user, pass, host, port, db_name)
            }
            None => format!("mongodb://{}:{}/{}?directConnection=true", host, port, db_name),
        };

        info!("Connecting to MongoDB database");
        let client = Client::with_uri_str(&uri).await.unwrap();
        let db = client.database(&db_name);

        let users: Collection<User> = db.collection("users");
        let repositories: Collection<Repository> = db.collection("repositories");
        let tokens: Collection<Token> = db.collection("tokens");

        Database { users, repositories, tokens }
    }
}
