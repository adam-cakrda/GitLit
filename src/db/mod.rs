mod helpers;
mod models;
pub mod exporters;

use mongodb::{Client, Collection};
use std::env;
use mongodb::bson::{doc, DateTime};
use serde::{Deserialize, Serialize};
use tracing::info;
use models::*;

#[derive(Clone, Debug)]
pub struct Database {
    pub users: Collection<User>,
    pub repositories: Collection<Repository>,
    pub tokens: Collection<Token>,
}

impl Database {
    pub async fn init() -> Self {
        let uri = match env::var("MONGO_URI") {
            Ok(v) => v.to_string(),
            Err(_) => "mongodb://localhost:27017/?directConnection=true".to_string(),
        };

        info!("Connecting to database");
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database("gitlit");

        let users: Collection<User> = db.collection("users");
        let repositories: Collection<Repository> = db.collection("repositories");
        let tokens: Collection<Token> = db.collection("tokens");

        Database {
            users,
            repositories,
            tokens,
        }
    }
}