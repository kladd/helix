//! Git integration implemented by shelling out to the `git` binary.
//!
//! This is deliberately gix-free: [`crate::DiffProviderRegistry`] keeps using
//! gix for the gutter diff, while the status / log / branch views drive `git`
//! as a subprocess instead. All invocations are non-interactive and
//! locale-stable so the machine-read output parses reliably.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{bail, Context, Result};
use tokio::process::Command;

#[cfg(test)]
mod test;

#[cfg(unix)]
const NULL_DEVICE: &str = "/dev/null";
#[cfg(not(unix))]
const NULL_DEVICE: &str = "NUL";

/// One entry of `git status --porcelain`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    /// The raw two-character status field, kept verbatim so the UI can show
    /// exactly what `git status -s` would.
    pub xy: [u8; 2],
    pub path: PathBuf,
    /// Set for renames/copies.
    pub orig_path: Option<PathBuf>,
}

impl StatusEntry {
    /// Whether the change is (at least partly) staged, i.e. the index column
    /// of the porcelain code is set.
    pub fn is_staged(&self) -> bool {
        self.xy[0] != b' ' && self.xy[0] != b'?'
    }
}

/// One line of `git log --oneline`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub hash: String,
    pub summary: String,
}

/// Resolved `HEAD`: a branch name, or a short hash when detached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Head {
    Branch(String),
    Detached(String),
}

/// Builds a `git` command rooted at `cwd`. The environment is pinned so
/// output is stable to parse: no pager, no interactive prompts, no optional
/// locks, and a fixed locale.
fn base(cwd: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(cwd)
        .arg("-c")
        .arg("core.pager=cat")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .env_remove("GIT_DIR")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    cmd
}

async fn run(cwd: &Path, args: &[&str]) -> Result<Vec<u8>> {
    run_codes(cwd, args, &[0]).await
}

/// Runs `git <args>`, treating any exit code in `ok` as success. `git diff
/// --no-index` returns 1 when differences exist, which is not an error here.
async fn run_codes(cwd: &Path, args: &[&str], ok: &[i32]) -> Result<Vec<u8>> {
    let out = base(cwd)
        .args(args)
        .output()
        .await
        .with_context(|| format!("failed to spawn `git {}`", args.join(" ")))?;
    let code = out.status.code().unwrap_or(-1);
    if !ok.contains(&code) {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail!(
            "git {} failed: {}",
            args.first().copied().unwrap_or_default(),
            stderr.trim()
        );
    }
    Ok(out.stdout)
}

#[cfg(unix)]
fn bytes_to_path(b: &[u8]) -> PathBuf {
    use std::os::unix::ffi::OsStrExt;
    std::ffi::OsStr::from_bytes(b).into()
}
#[cfg(not(unix))]
fn bytes_to_path(b: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(b).into_owned())
}

fn trimmed(out: &[u8]) -> String {
    String::from_utf8_lossy(out).trim().to_string()
}

/// `Err` here means the directory is not inside a git repository.
pub async fn repo_root(cwd: &Path) -> Result<PathBuf> {
    let out = run(cwd, &["rev-parse", "--show-toplevel"]).await?;
    let root = trimmed(&out);
    if root.is_empty() {
        bail!("not a git repository");
    }
    Ok(PathBuf::from(root))
}

async fn has_commits(cwd: &Path) -> bool {
    run(cwd, &["rev-parse", "--verify", "--quiet", "HEAD"])
        .await
        .is_ok()
}

pub async fn status(cwd: &Path) -> Result<Vec<StatusEntry>> {
    let out = run(cwd, &["status", "--porcelain=v1", "-z"]).await?;
    Ok(parse_status(&out))
}

fn parse_status(out: &[u8]) -> Vec<StatusEntry> {
    let mut entries = Vec::new();
    let mut iter = out.split(|&b| b == 0).filter(|t| !t.is_empty()).peekable();
    while let Some(tok) = iter.next() {
        if tok.len() < 3 {
            continue;
        }
        let xy = [tok[0], tok[1]];
        // With `-z`, a rename/copy is emitted as `XY <new> NUL <orig> NUL`,
        // so the original path is the next NUL-separated record.
        let is_rename = xy[0] == b'R' || xy[0] == b'C' || xy[1] == b'R' || xy[1] == b'C';
        let orig_path = if is_rename {
            iter.next().map(bytes_to_path)
        } else {
            None
        };
        entries.push(StatusEntry {
            xy,
            path: bytes_to_path(&tok[3..]),
            orig_path,
        });
    }
    entries
}

/// Falls back to the index when the repository has no commits yet, since
/// `HEAD` does not resolve in an unborn repo.
pub async fn file_diff(cwd: &Path, path: &Path) -> Result<String> {
    let p = path.to_string_lossy();
    let args: &[&str] = if has_commits(cwd).await {
        &["diff", "HEAD", "--", &p]
    } else {
        &["diff", "--", &p]
    };
    Ok(String::from_utf8_lossy(&run(cwd, args).await?).into_owned())
}

/// Untracked files have no tracked counterpart, so diff against the null
/// device to render them as all-additions. `--no-index` exits 1 when the
/// content differs, which is the expected case here.
pub async fn untracked_diff(cwd: &Path, path: &Path) -> Result<String> {
    let p = path.to_string_lossy();
    let out = run_codes(cwd, &["diff", "--no-index", "--", NULL_DEVICE, &p], &[0, 1]).await?;
    Ok(String::from_utf8_lossy(&out).into_owned())
}

pub async fn stage(cwd: &Path, path: &Path) -> Result<()> {
    run(cwd, &["add", "--", &path.to_string_lossy()]).await?;
    Ok(())
}

pub async fn unstage(cwd: &Path, path: &Path) -> Result<()> {
    run(cwd, &["restore", "--staged", "--", &path.to_string_lossy()]).await?;
    Ok(())
}

/// An unborn repository yields an empty list rather than an error.
pub async fn log(cwd: &Path) -> Result<Vec<LogEntry>> {
    if !has_commits(cwd).await {
        return Ok(Vec::new());
    }
    let out = run(cwd, &["log", "--no-color", "--pretty=format:%h%x09%s"]).await?;
    let log = String::from_utf8_lossy(&out);
    Ok(log
        .lines()
        .filter_map(|line| {
            let (hash, summary) = line.split_once('\t')?;
            Some(LogEntry {
                hash: hash.to_string(),
                summary: summary.to_string(),
            })
        })
        .collect())
}

pub async fn commit_diff(cwd: &Path, hash: &str) -> Result<String> {
    let out = run(cwd, &["show", "--no-color", hash]).await?;
    Ok(String::from_utf8_lossy(&out).into_owned())
}

pub async fn branches(cwd: &Path) -> Result<Vec<String>> {
    let out = run(
        cwd,
        &["for-each-ref", "--format=%(refname:short)", "refs/heads"],
    )
    .await?;
    Ok(String::from_utf8_lossy(&out)
        .lines()
        .map(|l| l.to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

pub async fn head(cwd: &Path) -> Result<Head> {
    if let Ok(out) = run(cwd, &["symbolic-ref", "--quiet", "--short", "HEAD"]).await {
        let name = trimmed(&out);
        if !name.is_empty() {
            return Ok(Head::Branch(name));
        }
    }
    let out = run(cwd, &["rev-parse", "--short", "HEAD"]).await?;
    Ok(Head::Detached(trimmed(&out)))
}

/// Errors (e.g. a dirty worktree that would be overwritten) are surfaced to
/// the caller, leaving the current branch unchanged.
pub async fn switch(cwd: &Path, name: &str) -> Result<()> {
    run(cwd, &["switch", name]).await?;
    Ok(())
}

/// Creates the branch from the current `HEAD` and switches to it.
pub async fn create_branch(cwd: &Path, name: &str) -> Result<()> {
    run(cwd, &["switch", "-c", name]).await?;
    Ok(())
}

/// One entry from `git blame --porcelain` — one per document line.
#[derive(Debug, Clone)]
pub struct BlameEntry {
    pub short_hash: String,
    pub author: String,
    /// Unix timestamp of the author date.
    pub time: i64,
}

/// Run `git blame --porcelain` for `path` over lines `[first_line, last_line]`
/// (0-indexed).  Returns one entry per line in the requested range.
/// Returns an empty vec for untracked files, new repos, or any git error.
pub async fn blame(
    cwd: &Path,
    path: &Path,
    first_line: usize,
    last_line: usize,
) -> Result<Vec<BlameEntry>> {
    if !has_commits(cwd).await {
        return Ok(Vec::new());
    }
    let p = path.to_string_lossy();
    let range = format!("{},{}", first_line + 1, last_line + 1);
    let out = match run(cwd, &["blame", "--porcelain", "-L", &range, "--", &p]).await {
        Ok(o) => o,
        Err(_) => return Ok(Vec::new()),
    };
    Ok(parse_blame(&out))
}

fn parse_blame(out: &[u8]) -> Vec<BlameEntry> {
    let text = String::from_utf8_lossy(out);
    let mut entries = Vec::new();
    let mut cache: std::collections::HashMap<String, (String, i64)> = std::collections::HashMap::new();

    let mut cur_hash = String::new();
    let mut cur_author: Option<String> = None;
    let mut cur_time: Option<i64> = None;

    for line in text.lines() {
        if line.starts_with('\t') {
            if cur_hash.is_empty() {
                continue;
            }
            let (author, time) = if let Some(cached) = cache.get(&cur_hash) {
                cached.clone()
            } else if let (Some(a), Some(t)) = (cur_author.take(), cur_time.take()) {
                cache.insert(cur_hash.clone(), (a.clone(), t));
                (a, t)
            } else {
                continue;
            };
            entries.push(BlameEntry {
                short_hash: cur_hash[..7.min(cur_hash.len())].to_string(),
                author,
                time,
            });
            cur_author = None;
            cur_time = None;
        } else if let Some(a) = line.strip_prefix("author ") {
            cur_author = Some(a.to_string());
        } else if let Some(t) = line.strip_prefix("author-time ") {
            cur_time = t.trim().parse().ok();
        } else {
            let bytes = line.as_bytes();
            if bytes.len() > 40 && bytes[40] == b' ' && bytes[..40].iter().all(|b| b.is_ascii_hexdigit()) {
                cur_hash = line[..40].to_string();
                if let Some((a, t)) = cache.get(&cur_hash) {
                    cur_author = Some(a.clone());
                    cur_time = Some(*t);
                } else {
                    cur_author = None;
                    cur_time = None;
                }
            }
        }
    }

    entries
}
