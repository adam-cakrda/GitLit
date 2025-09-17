use mongodb::{Client, Collection};
use crate::models::*;
use std::env;
use mongodb::results::InsertOneResult;
use mongodb::bson::doc;
use tracing::info;

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

    pub async fn create_user(&self, user: User) -> mongodb::error::Result<InsertOneResult> {
        let result = self
            .users
            .insert_one(user)
            .await
            .ok()
            .expect("Error creating user");

        info!("Created user: {:?}", result);
        Ok(result)
    }

    pub async fn create_repository(&self, repository: Repository) -> mongodb::error::Result<InsertOneResult> {
        let result = self
            .repositories
            .insert_one(repository)
            .await
            .ok()
            .expect("Error creating repository");

        info!("Created repository: {:?}", result);
        Ok(result)
    }

    pub async fn find_user_by_login(&self, login: &str) -> mongodb::error::Result<Option<User>> {
        use mongodb::bson::doc;
        let filter = doc! { "$or": [ { "username": login }, { "email": login } ] };
        self.users.find_one(filter).await
    }

    pub async fn create_token(&self, token: Token) -> mongodb::error::Result<InsertOneResult> {
        let result = self.tokens.insert_one(token).await?;
        info!("Created token: {:?}", result);
        Ok(result)
    }

    pub async fn find_token(&self, token_value: &str) -> mongodb::error::Result<Option<Token>> {
        self.tokens.find_one(doc! { "token": token_value }).await
    }

    pub async fn delete_token(&self, token_value: &str) -> mongodb::error::Result<u64> {
        let res = self.tokens.delete_one(doc! { "token": token_value }).await?;
        Ok(res.deleted_count)
    }

}