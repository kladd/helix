use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

use super::*;

/// Run a setup `git` command synchronously in `dir`, in an isolated
/// environment (mirrors `helix-vcs/src/git/test.rs`).
fn git(dir: &Path, args: &str) {
    let res = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args.split_whitespace())
        .env_remove("GIT_DIR")
        .env("GIT_TERMINAL_PROMPT", "false")
        .env("GIT_AUTHOR_DATE", "2000-01-01 00:00:00 +0000")
        .env("GIT_AUTHOR_EMAIL", "author@example.com")
        .env("GIT_AUTHOR_NAME", "author")
        .env("GIT_COMMITTER_DATE", "2000-01-02 00:00:00 +0000")
        .env("GIT_COMMITTER_EMAIL", "committer@example.com")
        .env("GIT_COMMITTER_NAME", "committer")
        .env("GIT_CONFIG_COUNT", "2")
        .env("GIT_CONFIG_KEY_0", "commit.gpgsign")
        .env("GIT_CONFIG_VALUE_0", "false")
        .env("GIT_CONFIG_KEY_1", "init.defaultBranch")
        .env("GIT_CONFIG_VALUE_1", "main")
        .output()
        .unwrap_or_else(|_| panic!("`git {args}` failed to spawn"));
    assert!(
        res.status.success(),
        "`git {args}` failed: {}",
        String::from_utf8_lossy(&res.stderr)
    );
}

fn write(path: &Path, contents: &str) {
    File::create(path)
        .unwrap()
        .write_all(contents.as_bytes())
        .unwrap();
}

fn empty_repo() -> TempDir {
    let tmp = tempfile::tempdir().unwrap();
    git(tmp.path(), "init");
    tmp
}

fn repo_with_commit() -> TempDir {
    let tmp = empty_repo();
    write(&tmp.path().join("tracked.txt"), "one\n");
    git(tmp.path(), "add -A");
    git(tmp.path(), "commit -m initial");
    tmp
}

#[test]
fn parse_status_handles_rename_and_untracked() {
    // `git status --porcelain -z`: NUL-separated records; a rename is
    // `R  <new> NUL <orig>`.
    let raw = b"R  new.txt\0old.txt\0?? fresh.txt\0 M tracked.txt\0";
    let entries = parse_status(raw);
    assert_eq!(entries.len(), 3);

    assert_eq!(&entries[0].xy, b"R ");
    assert_eq!(entries[0].path, Path::new("new.txt"));
    assert_eq!(entries[0].orig_path.as_deref(), Some(Path::new("old.txt")));
    assert!(entries[0].is_staged());

    assert_eq!(&entries[1].xy, b"??");
    assert_eq!(entries[1].path, Path::new("fresh.txt"));
    assert!(!entries[1].is_staged());

    assert_eq!(&entries[2].xy, b" M");
    assert!(!entries[2].is_staged());
}

#[tokio::test]
async fn repo_root_and_non_repo() {
    let repo = repo_with_commit();
    let root = repo_root(repo.path()).await.unwrap();
    assert_eq!(
        root.canonicalize().unwrap(),
        repo.path().canonicalize().unwrap()
    );

    let not_repo = tempfile::tempdir().unwrap();
    assert!(repo_root(not_repo.path()).await.is_err());
}

#[tokio::test]
async fn status_reports_changes_verbatim() {
    let repo = repo_with_commit();
    write(&repo.path().join("tracked.txt"), "one\ntwo\n");
    write(&repo.path().join("new.txt"), "new\n");

    let entries = status(repo.path()).await.unwrap();
    let tracked = entries
        .iter()
        .find(|e| e.path == Path::new("tracked.txt"))
        .unwrap();
    assert_eq!(&tracked.xy, b" M");
    assert!(!tracked.is_staged());

    git(repo.path(), "add new.txt");
    let entries = status(repo.path()).await.unwrap();
    let new = entries
        .iter()
        .find(|e| e.path == Path::new("new.txt"))
        .unwrap();
    assert_eq!(&new.xy, b"A ");
    assert!(new.is_staged());
}

#[tokio::test]
async fn head_branch_and_detached() {
    let repo = repo_with_commit();
    assert_eq!(
        head(repo.path()).await.unwrap(),
        Head::Branch("main".to_string())
    );

    git(repo.path(), "checkout --detach HEAD");
    assert!(matches!(
        head(repo.path()).await.unwrap(),
        Head::Detached(_)
    ));
}

#[tokio::test]
async fn branches_and_create_switch() {
    let repo = repo_with_commit();
    assert_eq!(branches(repo.path()).await.unwrap(), vec!["main"]);

    create_branch(repo.path(), "feature").await.unwrap();
    assert_eq!(
        head(repo.path()).await.unwrap(),
        Head::Branch("feature".to_string())
    );
    let mut names = branches(repo.path()).await.unwrap();
    names.sort();
    assert_eq!(names, vec!["feature", "main"]);

    switch(repo.path(), "main").await.unwrap();
    assert_eq!(
        head(repo.path()).await.unwrap(),
        Head::Branch("main".to_string())
    );
    assert!(switch(repo.path(), "does-not-exist").await.is_err());
}

#[tokio::test]
async fn log_lists_commits_and_empty_repo() {
    let empty = empty_repo();
    assert!(log(empty.path()).await.unwrap().is_empty());

    let repo = repo_with_commit();
    write(&repo.path().join("tracked.txt"), "two\n");
    git(repo.path(), "add -A");
    git(repo.path(), "commit -m second");

    let entries = log(repo.path()).await.unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].summary, "second");
    assert_eq!(entries[1].summary, "initial");
    assert!(!entries[0].hash.is_empty());
}

#[tokio::test]
async fn file_diff_and_untracked_diff() {
    let repo = repo_with_commit();
    write(&repo.path().join("tracked.txt"), "one\nchanged\n");
    let diff = file_diff(repo.path(), Path::new("tracked.txt"))
        .await
        .unwrap();
    assert!(diff.contains("+changed"), "diff was: {diff}");

    write(&repo.path().join("brand-new.txt"), "hello\n");
    let diff = untracked_diff(repo.path(), Path::new("brand-new.txt"))
        .await
        .unwrap();
    assert!(diff.contains("+hello"), "diff was: {diff}");
}

#[tokio::test]
async fn stage_and_unstage_roundtrip() {
    let repo = repo_with_commit();
    write(&repo.path().join("tracked.txt"), "one\nmore\n");

    stage(repo.path(), Path::new("tracked.txt")).await.unwrap();
    let entry = status(repo.path())
        .await
        .unwrap()
        .into_iter()
        .find(|e| e.path == Path::new("tracked.txt"))
        .unwrap();
    assert!(entry.is_staged(), "xy was {:?}", entry.xy);

    unstage(repo.path(), Path::new("tracked.txt"))
        .await
        .unwrap();
    let entry = status(repo.path())
        .await
        .unwrap()
        .into_iter()
        .find(|e| e.path == Path::new("tracked.txt"))
        .unwrap();
    assert!(!entry.is_staged(), "xy was {:?}", entry.xy);
}

#[tokio::test]
async fn commit_diff_shows_patch() {
    let repo = repo_with_commit();
    let entries = log(repo.path()).await.unwrap();
    let diff = commit_diff(repo.path(), &entries[0].hash).await.unwrap();
    assert!(diff.contains("tracked.txt"), "diff was: {diff}");
    assert!(diff.contains("+one"), "diff was: {diff}");
}
