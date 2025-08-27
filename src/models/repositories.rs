use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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
