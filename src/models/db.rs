use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Repository {
    pub _id: ObjectId,
    pub user: ObjectId, // owner
    pub name: String,
    pub description: String,
    pub is_private: bool,
    pub forked_from: Option<ObjectId>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    pub _id: ObjectId,
    pub user: ObjectId,
    pub token: String,
    pub created_at: DateTime,
    pub expires_at: Option<DateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub _id: ObjectId,
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub created_at: DateTime,
}