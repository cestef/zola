use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GitInfo {
    branch: String,
    commit_hash: String,
    tag: String,
    is_dirty: bool,
}

pub fn git_info() -> GitInfo {
    let repo = git2::Repository::discover(".").ok();
    if let Some(repo) = repo {
        let head = repo.head().ok();
        let branch = head
            .as_ref()
            .and_then(|h| h.shorthand().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());
        let commit = head
            .as_ref()
            .and_then(|h| h.peel_to_commit().ok())
            .map(|c| c.id().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let tag = repo
            .references()
            .ok()
            .and_then(|mut refs| {
                refs.find(|r| {
                    if let Ok(r) = r {
                        if let Some(name) = r.shorthand() {
                            return name.starts_with("refs/tags/");
                        }
                    }
                    false
                })
            })
            .and_then(|r| r.ok())
            .and_then(|r| r.shorthand().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());
        let is_dirty = repo.statuses(None).map_or(false, |statuses| {
            statuses.iter().any(|entry| {
                entry.status() != git2::Status::CURRENT && entry.status() != git2::Status::IGNORED
            })
        });
        GitInfo { branch, commit_hash: commit, tag, is_dirty }
    } else {
        GitInfo {
            branch: "unknown".to_string(),
            commit_hash: "unknown".to_string(),
            tag: "unknown".to_string(),
            is_dirty: false,
        }
    }
}
