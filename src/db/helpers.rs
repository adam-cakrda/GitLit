use mongodb::bson::doc;
use mongodb::results::InsertOneResult;
use tracing::info;
use crate::db;

impl super::Database {
    pub async fn create_user(&self, user: db::User) -> mongodb::error::Result<InsertOneResult> {
        let result = self
            .users
            .insert_one(user)
            .await?;

        info!("Created user: {:?}", result);
        Ok(result)
    }

    pub async fn create_repository(&self, repository: db::Repository) -> mongodb::error::Result<InsertOneResult> {
        let result = self
            .repositories
            .insert_one(repository)
            .await?;

        info!("Created repository: {:?}", result);
        Ok(result)
    }

    pub async fn find_user_by_login(&self, login: &str) -> mongodb::error::Result<Option<db::User>> {
        let filter = doc! { "$or": [ { "username": login }, { "email": login } ] };
        let res = self.users.find_one(filter).await?;
        Ok(res)
    }

    pub async fn find_user_by_id(&self, id: &bson::oid::ObjectId) -> mongodb::error::Result<Option<db::User>> {
        let res = self.users.find_one(doc! { "_id": id }).await?;
        Ok(res)
    }

    pub async fn create_token(&self, token: db::Token) -> mongodb::error::Result<InsertOneResult> {
        let result = self.tokens.insert_one(token).await?;
        info!("Created token: {:?}", result);
        Ok(result)
    }

    pub async fn find_repo(&self, id: &bson::oid::ObjectId) -> mongodb::error::Result<Option<db::Repository>> {
        let res = self.repositories.find_one(doc! { "_id": id }).await?;
        Ok(res)
    }

    pub async fn find_repo_by_hex(&self, id_hex: &str) -> mongodb::error::Result<Option<db::Repository>> {
        let oid = match bson::oid::ObjectId::parse_str(id_hex) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };
        self.find_repo(&oid).await
    }

    pub async fn is_repo_exists(&self, user_id: &bson::oid::ObjectId, name: &String) -> mongodb::error::Result<bool> {
        let filter = doc! { "user": user_id, "name": name };
        let res = self.repositories.find_one(filter).await?;
        Ok(res.is_some())
    }

    pub async fn find_repo_by_user_and_name(&self, user_id: &bson::oid::ObjectId, name: &str) -> mongodb::error::Result<Option<db::Repository>> {
        let filter = doc! { "user": user_id, "name": name };
        let res = self.repositories.find_one(filter).await?;
        Ok(res)
    }

    pub async fn find_token(&self, token_value: &str) -> mongodb::error::Result<Option<db::Token>> {
        let res = self.tokens.find_one(doc! { "token": token_value }).await?;
        Ok(res)
    }

    pub async fn delete_token(&self, token_value: &str) -> mongodb::error::Result<u64> {
        let res = self.tokens.delete_one(doc! { "token": token_value }).await?;
        Ok(res.deleted_count)
    }
    pub async fn delete_repository_by_id(&self, id: &bson::oid::ObjectId) -> mongodb::error::Result<u64> {
        let res = self.repositories.delete_one(doc! { "_id": id }).await?;
        Ok(res.deleted_count)
    }

    pub async fn find_repos_with_filter_sort(
        &self,
        filter: bson::Document,
        sort: bson::Document,
    ) -> mongodb::error::Result<Vec<db::Repository>> {
        use futures_util::TryStreamExt;
        let cursor = self.repositories.find(filter).sort(sort).await?;
        let repos: Vec<super::models::Repository> = cursor.try_collect().await?;
        Ok(repos)
    }
}
