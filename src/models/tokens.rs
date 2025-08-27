use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    pub _id: ObjectId,
    pub user: ObjectId,
    pub token: String,
    pub created_at: DateTime,
    pub expires_at: Option<DateTime>,
}