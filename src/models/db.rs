use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};
use mongodb::bson::serde_helpers::{
    serialize_bson_datetime_as_rfc3339_string,
    serialize_object_id_as_hex_string,
};

fn serialize_option_object_id_as_hex_string<S>(
    value: &Option<ObjectId>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(oid) => serialize_object_id_as_hex_string(oid, serializer),
        None => serializer.serialize_none(),
    }
}

fn serialize_option_bson_datetime_as_rfc3339_string<S>(
    value: &Option<DateTime>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(dt) => serialize_bson_datetime_as_rfc3339_string(dt, serializer),
        None => serializer.serialize_none(),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
pub struct Repository {
    #[serde(serialize_with = "serialize_object_id_as_hex_string")]
    pub _id: ObjectId,
    #[serde(serialize_with = "serialize_object_id_as_hex_string")]
    pub user: ObjectId, // owner
    pub name: String,
    pub description: String,
    pub is_private: bool,
    #[serde(serialize_with = "serialize_option_object_id_as_hex_string")]
    pub forked_from: Option<ObjectId>,
    #[serde(serialize_with = "serialize_bson_datetime_as_rfc3339_string")]
    pub created_at: DateTime,
    #[serde(serialize_with = "serialize_bson_datetime_as_rfc3339_string")]
    pub updated_at: DateTime,
}

#[derive(Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct Token {
    #[serde(serialize_with = "serialize_object_id_as_hex_string")]
    pub _id: ObjectId,
    #[serde(serialize_with = "serialize_object_id_as_hex_string")]
    pub user: ObjectId,
    pub token: String,
    #[serde(serialize_with = "serialize_bson_datetime_as_rfc3339_string")]
    pub created_at: DateTime,
    #[serde(serialize_with = "serialize_option_bson_datetime_as_rfc3339_string")]
    pub expires_at: Option<DateTime>,
}

#[derive(Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct User {
    #[serde(serialize_with = "serialize_object_id_as_hex_string")]
    pub _id: ObjectId,
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    #[serde(serialize_with = "serialize_bson_datetime_as_rfc3339_string")]
    pub created_at: DateTime,
}