use crate::db::models as dbm;
use crate::models as apim;

pub fn from_mongodb_to_user(user: dbm::User) -> apim::User {
    apim::User {
        _id: user._id.to_hex(),
        username: user.username,
        email: user.email,
        password: user.password,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        created_at: user.created_at.to_chrono(),
    }
}

pub fn from_user_to_mongodb(user: apim::User) -> dbm::User {
    dbm::User {
        _id: bson::oid::ObjectId::parse_str(&user._id).unwrap(),
        username: user.username.clone(),
        email: user.email.clone(),
        password: user.password.clone(),
        display_name: user.display_name.clone(),
        avatar_url: user.avatar_url.clone(),
        created_at: bson::DateTime::from_chrono(user.created_at),
    }
}

pub fn from_mongodb_to_repository(repository: dbm::Repository) -> apim::Repository {
    apim::Repository {
        _id: repository._id.to_hex(),
        user: repository.user.to_hex(),
        name: repository.name.clone(),
        description: repository.description.clone(),
        is_private: repository.is_private,
        forked_from: repository.forked_from.as_ref().map(|id| id.to_hex()),
        created_at: repository.created_at.to_chrono(),
        updated_at: repository.updated_at.to_chrono(),
    }
}

pub fn from_repository_to_mongodb(repository: apim::Repository) -> dbm::Repository {
    dbm::Repository {
        _id: bson::oid::ObjectId::parse_str(&repository._id).unwrap(),
        user: bson::oid::ObjectId::parse_str(&repository.user).unwrap(),
        name: repository.name.clone(),
        description: repository.description.clone(),
        is_private: repository.is_private,
        forked_from: match &repository.forked_from {
            Some(s) => Some(bson::oid::ObjectId::parse_str(s).unwrap()),
            None => None,
        },
        created_at: bson::DateTime::from_chrono(repository.created_at),
        updated_at: bson::DateTime::from_chrono(repository.updated_at),
    }
}

pub fn from_mongodb_to_token(token: dbm::Token) -> apim::Token {
    apim::Token {
        _id: token._id.to_hex(),
        user: token.user.to_hex(),
        token: token.token.clone(),
        created_at: token.created_at.to_chrono(),
        expires_at: token.expires_at.map(|dt| dt.to_chrono()),
    }
}

pub fn from_token_to_mongodb(token: apim::Token) -> dbm::Token {
    dbm::Token {
        _id: bson::oid::ObjectId::parse_str(&token._id).unwrap(),
        user: bson::oid::ObjectId::parse_str(&token.user).unwrap(),
        token: token.token.clone(),
        created_at: bson::DateTime::from_chrono(token.created_at),
        expires_at: token.expires_at.map(bson::DateTime::from_chrono),
    }
}