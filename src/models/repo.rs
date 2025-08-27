use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, utoipa::ToSchema)]
pub enum EntryKind {
    Blob,
    Tree,
    Commit,
    Other(String),
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct TreeEntry {
    pub mode: String,
    pub kind: EntryKind,
    pub oid: String,
    pub size: Option<u64>,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct CommitInfo {
    pub hash: String,
    pub name: String,
    pub email: String,
    pub timestamp_secs: i64,
    pub subject: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct Branch {
    pub name: String,
    pub oid: String,
    pub is_head: bool,
    pub upstream: Option<String>,
}