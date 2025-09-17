use mongodb::bson::doc;
use mongodb::results::InsertOneResult;
use tracing::info;
use crate::models as apim;
use super::exporters;

impl super::Database {
    pub async fn create_user(&self, user: apim::User) -> mongodb::error::Result<InsertOneResult> {
        let result = self
            .users
            .insert_one(exporters::from_user_to_mongodb(user))
            .await?;

        info!("Created user: {:?}", result);
        Ok(result)
    }

    pub async fn create_repository(&self, repository: apim::Repository) -> mongodb::error::Result<InsertOneResult> {
        let result = self
            .repositories
            .insert_one(exporters::from_repository_to_mongodb(repository))
            .await?;

        info!("Created repository: {:?}", result);
        Ok(result)
    }

    pub async fn find_user_by_login(&self, login: &str) -> mongodb::error::Result<Option<apim::User>> {
        let filter = doc! { "$or": [ { "username": login }, { "email": login } ] };
        let res = self.users.find_one(filter).await?;
        Ok(res.map(exporters::from_mongodb_to_user))
    }

    pub async fn find_user_by_id(&self, id: &str) -> mongodb::error::Result<Option<apim::User>> {
        let oid = match bson::oid::ObjectId::parse_str(id) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };
        let res = self.users.find_one(doc! { "_id": oid }).await?;
        Ok(res.map(exporters::from_mongodb_to_user))
    }

    pub async fn create_token(&self, token: apim::Token) -> mongodb::error::Result<InsertOneResult> {
        let result = self.tokens.insert_one(exporters::from_token_to_mongodb(token)).await?;
        info!("Created token: {:?}", result);
        Ok(result)
    }

    pub async fn find_repo(&self, id: String) -> mongodb::error::Result<Option<apim::Repository>> {
        let oid = match bson::oid::ObjectId::parse_str(&id) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };
        let res = self.repositories.find_one(doc! { "_id": oid }).await?;
        Ok(res.map(exporters::from_mongodb_to_repository))
    }

    pub async fn is_repo_exists(&self, user_id: &String, name: &String) -> mongodb::error::Result<bool> {
        let filter = doc! { "user": bson::oid::ObjectId::parse_str(user_id).unwrap(), "name": name };
        let res = self.repositories.find_one(filter).await?;
        Ok(res.is_some())
    }

    pub async fn find_repo_by_user_and_name(&self, user_id: &str, name: &str) -> mongodb::error::Result<Option<apim::Repository>> {
        let user_oid = match bson::oid::ObjectId::parse_str(user_id) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };
        let filter = doc! { "user": user_oid, "name": name };
        let res = self.repositories.find_one(filter).await?;
        Ok(res.map(exporters::from_mongodb_to_repository))
    }

    pub async fn find_token(&self, token_value: &str) -> mongodb::error::Result<Option<apim::Token>> {
        let res = self.tokens.find_one(doc! { "token": token_value }).await?;
        Ok(res.map(exporters::from_mongodb_to_token))
    }

    pub async fn delete_token(&self, token_value: &str) -> mongodb::error::Result<u64> {
        let res = self.tokens.delete_one(doc! { "token": token_value }).await?;
        Ok(res.deleted_count)
    }
    pub async fn delete_repository_by_id(&self, id: &str) -> mongodb::error::Result<u64> {
        let oid = match bson::oid::ObjectId::parse_str(id) {
            Ok(v) => v,
            Err(_) => return Ok(0),
        };
        let res = self.repositories.delete_one(doc! { "_id": oid }).await?;
        Ok(res.deleted_count)
    }

    pub async fn find_repos_with_filter_sort(
        &self,
        mut filter: bson::Document,
        sort: bson::Document,
    ) -> mongodb::error::Result<Vec<apim::Repository>> {
        use futures_util::TryStreamExt;
        let cursor = self.repositories.find(filter).sort(sort).await?;
        let repos_db: Vec<super::models::Repository> = cursor.try_collect().await?;
        let repos: Vec<apim::Repository> = repos_db
            .into_iter()
            .map(exporters::from_mongodb_to_repository)
            .collect();
        Ok(repos)
    }
}
