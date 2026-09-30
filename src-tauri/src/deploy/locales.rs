use crate::appstoreconnect::config::read_asc_config;
use crate::deploy::is_two_factor_prompt;
use crate::setup::{detect_kind, platform_dirs};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// Returned when the store has no app with this bundle ID / package name yet — the
/// generated `fetch_locales` lanes print `FASTLANE_APP_NOT_FOUND` for it.
pub const APP_NOT_FOUND: &str = "APP_NOT_FOUND";

fn strip_ansi(s: &str) -> String {
    Regex::new(r"\x1B\[[0-9;]*m").unwrap().replace_all(s, "").to_string()
}

/// Runs the `fastlane fetch_locales` lane and parses the `FASTLANE_LOCALES=...` line from stdout.
/// If not found, returns an error message (fastlane's [!] lines or the last few lines).
pub async fn run_fastlane_fetch_locales(fastlane_dir: &Path, workspace_path: &str) -> Result<Vec<String>, String> {
    if !fastlane_dir.exists() {
        return Err(format!("Folder not found: {}", fastlane_dir.display()));
    }

    let mut command = Command::new("fastlane");
    command.arg("fetch_locales").current_dir(fastlane_dir);

    // Authenticate with the App Store Connect API key when configured, instead of
    // fastlane's legacy Apple ID/app-specific-password session flow.
    if let Some(asc) = read_asc_config(workspace_path) {
        command
            .env("ASC_KEY_ID", asc.key_id)
            .env("ASC_ISSUER_ID", asc.issuer_id)
            .env("ASC_PRIVATE_KEY", asc.private_key);
    }

    // No dialog is reachable from this background prefetch (it can run before the deploy
    // event channel exists), so stdin is explicitly closed: if Apple asks for a 2-step
    // verification code here, fastlane can't get one and would otherwise hang until the
    // timeout below burns through a code that already reached the user's device for nothing.
    let mut child = match command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return Err(e.to_string()),
    };

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let read_fut = async {
        let mut out_lines: Vec<String> = Vec::new();
        let mut err_lines: Vec<String> = Vec::new();
        let mut two_factor = false;

        let mut out_reader = BufReader::new(stdout).lines();
        let mut err_reader = BufReader::new(stderr).lines();
        let mut out_done = false;
        let mut err_done = false;

        while !two_factor && !(out_done && err_done) {
            tokio::select! {
                line = out_reader.next_line(), if !out_done => match line {
                    Ok(Some(l)) => {
                        if is_two_factor_prompt(&l) { two_factor = true; }
                        out_lines.push(l);
                    }
                    _ => out_done = true,
                },
                line = err_reader.next_line(), if !err_done => match line {
                    Ok(Some(l)) => {
                        if is_two_factor_prompt(&l) { two_factor = true; }
                        err_lines.push(l);
                    }
                    _ => err_done = true,
                },
            }
        }

        (out_lines.join("\n"), err_lines.join("\n"), two_factor)
    };

    let result = tokio::time::timeout(Duration::from_secs(90), read_fut).await;
    let _ = child.kill().await;

    let Ok((stdout_text, _stderr_text, two_factor)) = result else {
        return Err("Timed out (90s)".to_string());
    };

    if two_factor {
        return Err("Apple 2-step verification required — skipped during automatic locale detection (no code entry available here)".to_string());
    }

    if stdout_text.contains("FASTLANE_APP_NOT_FOUND") {
        return Err(APP_NOT_FOUND.to_string());
    }

    let re = Regex::new(r"FASTLANE_LOCALES=(.+)").unwrap();
    if let Some(c) = re.captures(&stdout_text) {
        let locales: Vec<String> = c[1].trim().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        if !locales.is_empty() {
            return Ok(locales);
        }
    }

    let clean = strip_ansi(&stdout_text);
    let error_lines: Vec<&str> = clean
        .lines()
        .filter(|l| l.contains("[!]") || l.contains("Error") || l.contains("error") || l.contains("failed"))
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    let error = if !error_lines.is_empty() {
        error_lines.join(" | ")
    } else {
        let tail: Vec<&str> = clean.lines().filter(|l| !l.trim().is_empty()).collect();
        let last3: Vec<&str> = tail.iter().rev().take(3).rev().copied().collect();
        if last3.is_empty() { "Unknown error".to_string() } else { last3.join(" | ") }
    };

    Err(error)
}

/// Fetches the (iOS, Android) locales the app actually has in each store. Deliberately no
/// fallback: guessing (fastlane metadata folders, serconf.dart flags, a default `tr`) produced
/// locales the app didn't have, and deliver/supply then created them as new store languages.
pub async fn get_store_locales(workspace_path: &str) -> (Result<Vec<String>, String>, Result<Vec<String>, String>) {
    let root = Path::new(workspace_path);
    let (ios_dir, android_dir) = detect_kind(root).map(|kind| platform_dirs(root, kind)).unwrap_or((None, None));
    let fetch = |dir: Option<PathBuf>| async move {
        match dir {
            Some(dir) => run_fastlane_fetch_locales(&dir, workspace_path).await,
            None => Err("The project has no such platform".to_string()),
        }
    };
    tokio::join!(fetch(ios_dir), fetch(android_dir))
}
