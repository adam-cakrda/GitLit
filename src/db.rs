use mongodb::{Client, Collection};
use crate::models::*;
use std::env;
use mongodb::results::InsertOneResult;

pub struct Database {
    pub users: Collection<User>,
    pub repositories: Collection<Repository>,
}

impl Database {
    pub async fn init() -> Self {
        let uri = match env::var("MONGO_URI") {
            Ok(v) => v.to_string(),
            Err(_) => "mongodb://localhost:27017/?directConnection=true".to_string(),
        };

        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database("gitlit");

        let users: Collection<User> = db.collection("users");
        let repositories: Collection<Repository> = db.collection("repositories");

        Database {
            users,
            repositories,
        }
    }

    pub async fn create_user(&self, user: User) -> Result<InsertOneResult, mongodb::error::Error> {
        let result = self
            .users
            .insert_one(user)
            .await
            .ok()
            .expect("Error creating user");
        Ok(result)
    }

   pub async fn create_repository(&self, repository: Repository) -> Result<InsertOneResult, mongodb::error::Error> {
       let result = self
            .repositories
            .insert_one(repository)
            .await
            .ok()
            .expect("Error creating repository");
       Ok(result)
   }
}