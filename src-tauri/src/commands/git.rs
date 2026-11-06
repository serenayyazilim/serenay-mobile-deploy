//! The workspace's git branch, for the sidebar and the MCP server: which one it is on, and
//! switching to another one with its latest commits and packages.

use serde::Serialize;
use std::path::Path;
use tauri::{AppHandle, Emitter};
use tokio::process::Command;

async fn git(workspace: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git").args(args).current_dir(workspace).output().await.map_err(|e| format!("git: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    branch: String,
    /// Local branches and the remote ones not checked out yet, sorted.
    branches: Vec<String>,
    /// Files with uncommitted changes, untracked ones included.
    changed: usize,
    /// Commits ahead of / behind the upstream, as of the last fetch.
    ahead: u32,
    behind: u32,
}

#[tauri::command]
pub async fn git_status(workspace_path: String) -> Result<GitStatus, String> {
    let ws = workspace_path.as_str();
    let branch = git(ws, &["branch", "--show-current"]).await?;
    let local = git(ws, &["for-each-ref", "--format=%(refname:lstrip=2)", "refs/heads"]).await?;
    let remote = git(ws, &["for-each-ref", "--format=%(refname:lstrip=3)", "refs/remotes"]).await?;
    let mut branches: Vec<String> = local.lines().chain(remote.lines()).filter(|b| !b.is_empty() && *b != "HEAD").map(str::to_string).collect();
    branches.sort();
    branches.dedup();
    let changed = git(ws, &["status", "--porcelain"]).await?.lines().count();
    let counts = git(ws, &["rev-list", "--left-right", "--count", "HEAD...@{upstream}"]).await.unwrap_or_default();
    let mut counts = counts.split_whitespace().map(|n| n.parse().unwrap_or(0));
    let (ahead, behind) = (counts.next().unwrap_or(0), counts.next().unwrap_or(0));
    Ok(GitStatus { branch, branches, changed, ahead, behind })
}

/// Fetches, puts uncommitted changes aside in a stash, switches to `branch`, fast-forwards it to
/// its upstream and, for Flutter, runs `flutter pub get`. Returns what was done.
#[tauri::command]
pub async fn git_switch(app: AppHandle, workspace_path: String, branch: String) -> Result<String, String> {
    let result = switch(&workspace_path, &branch).await;
    let _ = app.emit("git-switched", ());
    result
}

async fn switch(ws: &str, branch: &str) -> Result<String, String> {
    let branch = branch.to_string();
    if branch.is_empty() || branch.starts_with('-') {
        return Err("Invalid branch name".to_string());
    }
    git(ws, &["fetch", "--prune"]).await.map_err(|e| format!("git fetch failed: {e}"))?;

    let mut done = Vec::new();
    let changed = git(ws, &["status", "--porcelain"]).await?.lines().count();
    if changed > 0 {
        let name = format!("serenay-deploy: before switching to {branch}");
        git(ws, &["stash", "push", "--include-untracked", "-m", &name]).await.map_err(|e| format!("git stash failed: {e}"))?;
        done.push(format!("{changed} changed files were put aside as the stash \"{name}\" (git stash pop brings them back)."));
    }
    if git(ws, &["branch", "--show-current"]).await? != branch {
        git(ws, &["switch", &branch]).await.map_err(|e| format!("git switch failed: {e}"))?;
    }
    if git(ws, &["rev-parse", "--abbrev-ref", "@{upstream}"]).await.is_ok() {
        git(ws, &["pull", "--ff-only"])
            .await
            .map_err(|e| format!("git pull failed (the branch has local commits not on the remote?): {e}"))?;
    }
    let head = git(ws, &["log", "-1", "--format=%h %s"]).await.unwrap_or_default();
    done.insert(0, format!("On {branch}, up to date: {head}."));

    if Path::new(ws).join("pubspec.yaml").exists() {
        let out = Command::new("flutter").args(["pub", "get"]).current_dir(ws).output().await.map_err(|e| format!("flutter pub get: {e}"))?;
        if !out.status.success() {
            return Err(format!("Switched, but flutter pub get failed:\n{}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        done.push("Packages fetched (flutter pub get).".to_string());
    }
    Ok(done.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn switches_with_changes_stashed_and_pulls() {
        let root = std::env::temp_dir().join(format!("serenay-git-{}", uuid::Uuid::new_v4()));
        let (origin, ws) = (root.join("origin"), root.join("ws"));
        std::fs::create_dir_all(&origin).unwrap();
        let o = origin.to_str().unwrap();
        for args in [&["init", "-q", "-b", "main"][..], &["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "first"], &["branch", "develop"]] {
            git(o, args).await.unwrap();
        }
        git(root.to_str().unwrap(), &["clone", "-q", "-b", "develop", o, "ws"]).await.unwrap();
        git(o, &["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "second"]).await.unwrap();
        let w = ws.to_str().unwrap();
        std::fs::write(ws.join("generated.txt"), "x").unwrap();

        let status = git_status(w.to_string()).await.unwrap();
        assert_eq!((status.branch.as_str(), status.branches, status.changed), ("develop", vec!["develop".to_string(), "main".to_string()], 1));

        let message = switch(w, "main").await.unwrap();
        assert!(message.starts_with("On main, up to date: ") && message.contains("second") && message.contains("1 changed files"), "{message}");
        assert_eq!(git_status(w.to_string()).await.unwrap().changed, 0);
        assert!(git(w, &["stash", "list"]).await.unwrap().contains("before switching to main"));
        assert!(switch(w, "-x").await.is_err());
        std::fs::remove_dir_all(&root).ok();
    }
}
