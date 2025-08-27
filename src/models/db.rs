use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
pub struct Repository {
    #[schema(value_type = String, example = "665f1f5a2ab79b9f2ff6a0d1")]
    pub _id: ObjectId,
    #[schema(value_type = String, example = "665f1f5a2ab79b9f2ff6a0d0")]
    pub user: ObjectId, // owner
    pub name: String,
    pub description: String,
    pub is_private: bool,
    #[schema(value_type = Option<String>)]
    pub forked_from: Option<ObjectId>,
    #[schema(value_type = String, format = DateTime, example = "2024-01-01T12:00:00Z")]
    pub created_at: DateTime,
    #[schema(value_type = String, format = DateTime, example = "2024-01-01T12:00:00Z")]
    pub updated_at: DateTime,
}

#[derive(Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct Token {
    #[schema(value_type = String)]
    pub _id: ObjectId,
    #[schema(value_type = String)]
    pub user: ObjectId,
    pub token: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub expires_at: Option<DateTime>,
}

#[derive(Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct User {
    #[schema(value_type = String)]
    pub _id: ObjectId,
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime,
}