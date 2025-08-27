use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRepoRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_private: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct OkResponse {
    pub ok: bool,
}

#[derive(Debug, Deserialize)]
pub struct ReposQuery {
    pub owner: Option<String>,
    // owner, newest, updated
    pub filter: Option<String>,
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BranchesQuery {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct BranchesResponse {
    pub branches: Vec<crate::models::Branch>,
}

#[derive(Debug, Deserialize)]
pub struct ContentQuery {
    pub id: String,
    pub path: Option<String>,
    pub branch: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ContentResponse {
    Tree { entries: Vec<crate::models::TreeEntry> },
    Blob { content_base64: String },
}

#[derive(Debug, Deserialize)]
pub struct CommitsQuery {
    pub id: String,
    pub branch: Option<String>,
    pub limit: Option<usize>,
}