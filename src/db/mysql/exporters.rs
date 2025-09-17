use crate::models as apim;
use chrono::{DateTime, Utc};
use sqlx::{mysql::MySqlRow, Row};

pub(crate) fn row_to_user(row: MySqlRow) -> apim::User {
    apim::User {
        _id: row.get::<String, _>("id"),
        username: row.get::<String, _>("username"),
        email: row.get::<String, _>("email"),
        password: row.get::<String, _>("password"),
        display_name: row.get::<String, _>("display_name"),
        avatar_url: row.try_get::<String, _>("avatar_url").ok(),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
    }
}

pub(crate) fn row_to_repo(row: MySqlRow) -> apim::Repository {
    apim::Repository {
        _id: row.get::<String, _>("id"),
        user: row.get::<String, _>("user_id"),
        name: row.get::<String, _>("name"),
        description: row.get::<String, _>("description"),
        is_private: row.get::<bool, _>("is_private"),
        forked_from: row.try_get::<String, _>("forked_from").ok(),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
        updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    }
}

pub(crate) fn row_to_token(row: MySqlRow) -> apim::Token {
    apim::Token {
        _id: row.get::<String, _>("id"),
        user: row.get::<String, _>("user_id"),
        token: row.get::<String, _>("token"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
        expires_at: row.try_get::<DateTime<Utc>, _>("expires_at").ok(),
    }
}
