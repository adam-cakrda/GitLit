use super::Database;
use crate::models as apim;
use super::exporters::{row_to_repo, row_to_token, row_to_user};
use bson::{Bson, Document};

// Helpers to build dynamic SQL parts for repository search
fn parse_top_level_filters(filter: &Document, where_clauses: &mut Vec<String>, params: &mut Vec<String>) {
    if let Some(user_val) = filter.get("user") {
        if let Bson::ObjectId(oid) = user_val {
            where_clauses.push("user_id = ?".to_string());
            params.push(oid.to_hex());
        }
    }
    if let Some(Bson::Boolean(is_priv)) = filter.get("is_private") {
        where_clauses.push("is_private = ?".to_string());
        params.push(if *is_priv { "1".into() } else { "0".into() });
    }
}

fn build_or_clause(filter: &Document, where_clauses: &mut Vec<String>, params: &mut Vec<String>) {
    let Some(Bson::Array(arr)) = filter.get("$or") else { return; };

    let mut or_subclauses: Vec<String> = Vec::new();
    for item in arr {
        if let Bson::Document(doc) = item {
            if let Some(Bson::Boolean(b)) = doc.get("is_private") {
                if let Some(Bson::ObjectId(uid)) = doc.get("user") {
                    or_subclauses.push("(is_private = ? AND user_id = ?)".into());
                    params.push(if *b { "1".into() } else { "0".into() });
                    params.push(uid.to_hex());
                } else {
                    or_subclauses.push("(is_private = ?)".into());
                    params.push(if *b { "1".into() } else { "0".into() });
                }
            } else if let Some(Bson::Document(re)) = doc.get("name") {
                if let Some(Bson::String(q)) = re.get("$regex") {
                    or_subclauses.push("(name LIKE ? )".into());
                    params.push(format!("%{}%", q));
                }
            } else if let Some(Bson::Document(re)) = doc.get("description") {
                if let Some(Bson::String(q)) = re.get("$regex") {
                    or_subclauses.push("(description LIKE ? )".into());
                    params.push(format!("%{}%", q));
                }
            }
        }
    }
    if !or_subclauses.is_empty() {
        where_clauses.push(format!("({})", or_subclauses.join(" OR ")));
    }
}

fn build_order_by(sort: &Document) -> String {
    if let Some(Bson::Int32(v)) = sort.get("created_at") {
        return if *v < 0 { "created_at DESC" } else { "created_at ASC" }.to_string();
    }
    if let Some(Bson::Int32(v)) = sort.get("updated_at") {
        return if *v < 0 { "updated_at DESC" } else { "updated_at ASC" }.to_string();
    }
    "updated_at DESC".to_string()
}

impl Database {
    pub async fn create_user(&self, user: apim::User) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO users (id, username, email, password, display_name, avatar_url, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(user._id)
        .bind(user.username)
        .bind(user.email)
        .bind(user.password)
        .bind(user.display_name)
        .bind(user.avatar_url)
        .bind(user.created_at)
        .execute(&self.pool)
        .await?
        ;
        Ok(())
    }

    pub async fn create_repository(&self, repo: apim::Repository) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO repositories (id, user_id, name, description, is_private, forked_from, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(repo._id)
        .bind(repo.user)
        .bind(repo.name)
        .bind(repo.description)
        .bind(repo.is_private)
        .bind(repo.forked_from)
        .bind(repo.created_at)
        .bind(repo.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_user_by_login(&self, login: &str) -> Result<Option<apim::User>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, username, email, password, display_name, avatar_url, created_at
             FROM users WHERE username = ? OR email = ? LIMIT 1",
        )
        .bind(login)
        .bind(login)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(row_to_user))
    }

    pub async fn find_user_by_id(&self, id: &str) -> Result<Option<apim::User>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, username, email, password, display_name, avatar_url, created_at
             FROM users WHERE id = ? LIMIT 1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_user))
    }

    pub async fn create_token(&self, token: apim::Token) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO tokens (id, user_id, token, created_at, expires_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(token._id)
        .bind(token.user)
        .bind(token.token)
        .bind(token.created_at)
        .bind(token.expires_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_repo(&self, id: String) -> Result<Option<apim::Repository>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, user_id, name, description, is_private, forked_from, created_at, updated_at
             FROM repositories WHERE id = ? LIMIT 1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_repo))
    }

    pub async fn is_repo_exists(&self, user_id: &String, name: &String) -> Result<bool, sqlx::Error> {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) as count FROM repositories WHERE user_id = ? AND name = ?",
        )
        .bind(user_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
    }

    pub async fn find_repo_by_user_and_name(&self, user_id: &str, name: &str) -> Result<Option<apim::Repository>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, user_id, name, description, is_private, forked_from, created_at, updated_at
             FROM repositories WHERE user_id = ? AND name = ? LIMIT 1",
        )
        .bind(user_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_repo))
    }

    pub async fn find_token(&self, token_value: &str) -> Result<Option<apim::Token>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, user_id, token, created_at, expires_at FROM tokens WHERE token = ? LIMIT 1",
        )
        .bind(token_value)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(row_to_token))
    }

    pub async fn delete_token(&self, token_value: &str) -> Result<u64, sqlx::Error> {
        let res = sqlx::query("DELETE FROM tokens WHERE token = ?")
            .bind(token_value)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

    pub async fn delete_repository_by_id(&self, id: &str) -> Result<u64, sqlx::Error> {
        let res = sqlx::query("DELETE FROM repositories WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

    pub async fn find_repos_with_filter_sort(&self, filter: bson::Document, sort: bson::Document) -> Result<Vec<apim::Repository>, sqlx::Error> {
        let mut where_clauses: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        parse_top_level_filters(&filter, &mut where_clauses, &mut params);
        build_or_clause(&filter, &mut where_clauses, &mut params);

        let order_by = build_order_by(&sort);

        let mut sql = String::from(
            "SELECT id, user_id, name, description, is_private, forked_from, created_at, updated_at FROM repositories",
        );
        if !where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clauses.join(" AND "));
        }
        sql.push_str(" ORDER BY ");
        sql.push_str(&order_by);

        let mut query = sqlx::query(&sql);
        for val in params {
            query = query.bind(val);
        }

        let rows = query.fetch_all(&self.pool).await?;
        let repos = rows.into_iter().map(row_to_repo).collect();
        Ok(repos)
    }
}
