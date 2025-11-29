use std::path::{PathBuf};
use crate::errors::GitError;
use crate::models::*;
use git2::{BranchType, ObjectType, Repository, Sort};
use bson::oid::ObjectId;

pub fn repo_path(user_id: &ObjectId, repo_id: &ObjectId) -> PathBuf {
    PathBuf::from("./repos").join(user_id.to_string()).join(repo_id.to_string())
}

pub async fn init(user_id: ObjectId, repo_id: ObjectId) -> Result<PathBuf, GitError> {
    let path = repo_path(&user_id, &repo_id);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    Repository::init_bare(&path).map_err(|e| GitError::Git(e.to_string()))?;
    Ok(path)
}

pub async fn list_branches(
    user_id: &ObjectId,
    repo_id: &ObjectId,
) -> Result<Vec<Branch>, GitError> {
    let repo_path = repo_path(user_id, repo_id);
    let repo = Repository::open_bare(&repo_path).map_err(|e| GitError::Git(e.to_string()))?;

    let mut out = Vec::new();
    let iter = repo
        .branches(Some(BranchType::Local))
        .map_err(|e| GitError::Git(e.to_string()))?;

    for bres in iter {
        let (branch, _ty) = bres.map_err(|e| GitError::Git(e.to_string()))?;

        let name = branch
            .name_bytes()
            .map(|b| String::from_utf8_lossy(b).to_string())
            .unwrap_or_else(|_| String::from("<invalid-utf8>"));


        let oid = branch
            .get()
            .target()
            .map(|o| o.to_string())
            .unwrap_or_default();

        let is_head = branch.is_head();


        let upstream = match branch.upstream() {
            Ok(up) => up
                .name_bytes()
                .map(|b| Some(String::from_utf8_lossy(b).to_string()))
                .unwrap_or(None),
            Err(_) => None,
        };

        out.push(Branch {
            name,
            oid,
            is_head,
            upstream,
        });
    }

    Ok(out)
}

pub async fn delete_branch(
    user_id: &ObjectId,
    repo_id: &ObjectId,
    branch_name: &String,
) -> Result<(), GitError> {
    let repo_path = repo_path(user_id, repo_id);
    let repo = Repository::open_bare(&repo_path).map_err(|e| GitError::Git(e.to_string()))?;

    let mut branch = repo
        .find_branch(branch_name, BranchType::Local)
        .map_err(|e| GitError::Git(format!("failed to find branch '{}': {}", branch_name, e)))?;

    if branch.is_head() {
        return Err(GitError::Git(format!(
            "cannot delete branch '{}': it is the current HEAD",
            branch_name
        )));
    }

    branch
        .delete()
        .map_err(|e| GitError::Git(format!("failed to delete branch '{}': {}", branch_name, e)))?;

    Ok(())
}

pub async fn list_commits(
    user_id: &ObjectId,
    repo_id: &ObjectId,
    reference: &str,
    branch: Option<&str>,
    limit: usize,
) -> Result<Vec<CommitInfo>, GitError> {
    let repo_path = repo_path(user_id, repo_id);
    let repo = Repository::open_bare(&repo_path).map_err(|e| GitError::Git(e.to_string()))?;

    let revspec = branch.unwrap_or(reference);

    let obj = repo
        .revparse_single(revspec)
        .map_err(|e| GitError::Git(e.to_string()))?;

    let start_id = obj.id();
    let mut walk = repo.revwalk().map_err(|e| GitError::Git(e.to_string()))?;
    walk
        .set_sorting(Sort::TIME | Sort::TOPOLOGICAL)
        .map_err(|e| GitError::Git(e.to_string()))?;
    walk.push(start_id).map_err(|e| GitError::Git(e.to_string()))?;

    let mut commits = Vec::new();
    let take_n = if limit == 0 { usize::MAX } else { limit };

    for oid_res in walk.take(take_n) {
        let oid = oid_res.map_err(|e| GitError::Git(e.to_string()))?;
        let commit = match repo.find_commit(oid) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let author = commit.author();
        let name = author.name().unwrap_or("").to_string();
        let email = author.email().unwrap_or("").to_string();
        let timestamp_secs = commit.time().seconds();
        let subject = commit.summary().unwrap_or("").to_string();

        commits.push(CommitInfo {
            hash: commit.id().to_string(),
            name,
            email,
            timestamp_secs,
            subject,
        });
    }

    Ok(commits)
}

pub async fn list_tree(
    user_id: &ObjectId,
    repo_id: &ObjectId,
    rev: &str,
    branch: Option<&str>,
    path: Option<&str>,
) -> Result<Vec<TreeEntry>, GitError> {
    let repo_path = repo_path(user_id, repo_id);
    let repo = Repository::open_bare(&repo_path).map_err(|e| GitError::Git(e.to_string()))?;

    let base = branch.unwrap_or(rev);
    let spec = if let Some(p) = path {
        format!("{}:{}", base, p)
    } else {
        format!("{}^{{tree}}", base)
    };

    let obj = repo
        .revparse_single(&spec)
        .map_err(|e| GitError::Git(e.to_string()))?;
    let tree = obj
        .peel_to_tree()
        .map_err(|e| GitError::Git(e.to_string()))?;

    let mut entries = Vec::new();
    for entry in tree.iter() {
        let mode = format!("{:06o}", entry.filemode() as u32);
        let oid = entry.id().to_string();
        let kind = match entry.kind() {
            Some(ObjectType::Blob) => EntryKind::Blob,
            Some(ObjectType::Tree) => EntryKind::Tree,
            Some(ObjectType::Commit) => EntryKind::Commit,
            Some(other) => EntryKind::Other(format!("{:?}", other)),
            None => EntryKind::Other("unknown".to_string()),
        };

        let size = if matches!(kind, EntryKind::Blob) {
            match repo.find_blob(entry.id()) {
                Ok(blob) => Some(blob.size() as u64),
                Err(_) => None,
            }
        } else {
            None
        };

        let path_str = entry
            .name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| String::from("<invalid-utf8>"));

        entries.push(TreeEntry {
            mode,
            kind,
            oid,
            size,
            path: path_str,
        });
    }

    Ok(entries)
}

pub async fn get_file_content(
    user_id: &ObjectId,
    repo_id: &ObjectId,
    rev: &str,
    branch: Option<&str>,
    path: &str,
) -> Result<Vec<u8>, GitError> {
    let repo_path = repo_path(user_id, repo_id);
    let repo = Repository::open_bare(&repo_path).map_err(|e| GitError::Git(e.to_string()))?;

    let base = branch.unwrap_or(rev);
    let spec = format!("{}:{}", base, path);
    let obj = repo
        .revparse_single(&spec)
        .map_err(|e| GitError::Git(e.to_string()))?;
    let blob = obj
        .peel_to_blob()
        .map_err(|e| GitError::Git(e.to_string()))?;

    Ok(blob.content().to_vec())
}

pub async fn commit_diff(
    user_id: &ObjectId,
    repo_id: &ObjectId,
    commit_hash: &str,
) -> Result<String, GitError> {
    use git2::{Oid, DiffOptions, DiffFormat};

    let repo_path = repo_path(user_id, repo_id);
    let repo = Repository::open_bare(&repo_path).map_err(|e| GitError::Git(e.to_string()))?;

    let oid = Oid::from_str(commit_hash).map_err(|e| GitError::Git(e.to_string()))?;
    let commit = repo.find_commit(oid).map_err(|e| GitError::Git(e.to_string()))?;
    let tree = commit.tree().map_err(|e| GitError::Git(e.to_string()))?;

    let parent_tree = if commit.parent_count() > 0 {
        let p = commit.parent(0).map_err(|e| GitError::Git(e.to_string()))?;
        Some(p.tree().map_err(|e| GitError::Git(e.to_string()))?)
    } else {
        None
    };

    let mut diff_opts = DiffOptions::new();
    let diff = match parent_tree {
        Some(ref pt) => repo.diff_tree_to_tree(Some(pt), Some(&tree), Some(&mut diff_opts))
            .map_err(|e| GitError::Git(e.to_string()))?,
        None => repo.diff_tree_to_tree(None, Some(&tree), Some(&mut diff_opts))
            .map_err(|e| GitError::Git(e.to_string()))?,
    };

    let mut buf = String::new();
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        use std::fmt::Write as _;
        let _ = match line.origin() {
            ' ' | '+' | '-' | '@' | 'F' => write!(&mut buf, "{}", std::str::from_utf8(line.content()).unwrap_or("")),
            _ => write!(&mut buf, "{}", std::str::from_utf8(line.content()).unwrap_or("")),
        };
        true
    }).map_err(|e| GitError::Git(e.to_string()))?;

    Ok(buf)
}

pub async fn collect_files_at_path(
    user_id: &ObjectId,
    repo_id: &ObjectId,
    rev: &str,
    branch: Option<&str>,
    path: Option<&str>,
) -> Result<Vec<(String, Vec<u8>)>, GitError> {
    use std::path::Path;

    let repo_path = repo_path(user_id, repo_id);
    let repo = Repository::open_bare(&repo_path).map_err(|e| GitError::Git(e.to_string()))?;

    let base = branch.unwrap_or(rev);
    let spec = if let Some(p) = path {
        format!("{}:{}", base, p)
    } else {
        format!("{}^{{tree}}", base)
    };

    let obj = repo
        .revparse_single(&spec)
        .map_err(|e| GitError::Git(e.to_string()))?;
    
    if let Ok(blob) = obj.peel_to_blob() {
        let name = if let Some(p) = path {
            Path::new(p)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("file")
                .to_string()
        } else {
            format!("{}.bin", obj.id())
        };
        return Ok(vec![(name, blob.content().to_vec())]);
    }
    
    let tree = obj
        .peel_to_tree()
        .map_err(|e| GitError::Git(e.to_string()))?;

    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    
    let root_prefix = if let Some(p) = path {
        let s = Path::new(p).file_name().and_then(|s| s.to_str()).unwrap_or("root");
        format!("{}/", s)
    } else {
        String::new()
    };

    tree.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
        if let Some(ObjectType::Blob) = entry.kind() {
            let file_rel = match entry.name() {
                Some(n) => {
                    if dir.is_empty() {
                        n.to_string()
                    } else {
                        format!("{}{}", dir, n)
                    }
                }
                None => return 0,
            };
            if let Ok(blob) = repo.find_blob(entry.id()) {
                let archive_path = format!("{}{}", root_prefix, file_rel);
                files.push((archive_path, blob.content().to_vec()));
            }
        }
        0
    }).map_err(|e| GitError::Git(e.to_string()))?;

    Ok(files)
}

#[allow(dead_code)]
pub async fn exists(user_id: &ObjectId, repo_id: &ObjectId) -> bool {
    let p = repo_path(user_id, repo_id);
    Repository::open_bare(&p).is_ok()
}