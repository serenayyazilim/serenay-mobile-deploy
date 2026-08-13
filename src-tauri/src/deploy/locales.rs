use crate::deploy::is_two_factor_prompt;
use regex::Regex;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

const FLAG_TO_LOCALES: &[(&str, &str, &str)] = &[
    ("ENGLISH", "en-US", "en-US"),
    ("RUSSIAN", "ru", "ru-RU"),
    ("FRENCH", "fr-FR", "fr-FR"),
    ("ITALIAN", "it", "it-IT"),
    ("ARABIC", "ar-SA", "ar"),
    ("SPANISH", "es-ES", "es-ES"),
    ("KAZAKH", "kk", "kk"),
];

fn strip_ansi(s: &str) -> String {
    Regex::new(r"\x1B\[[0-9;]*m").unwrap().replace_all(s, "").to_string()
}

/// Runs the `fastlane fetch_locales` lane and parses the `FASTLANE_LOCALES=...` line from stdout.
/// If not found, returns an error message (fastlane's [!] lines or the last few lines).
pub async fn run_fastlane_fetch_locales(fastlane_dir: &Path) -> Result<Vec<String>, String> {
    if !fastlane_dir.exists() {
        return Err(format!("Folder not found: {}", fastlane_dir.display()));
    }

    // No dialog is reachable from this background prefetch (it can run before the deploy
    // event channel exists), so stdin is explicitly closed: if Apple asks for a 2-step
    // verification code here, fastlane can't get one and would otherwise hang until the
    // timeout below burns through a code that already reached the user's device for nothing.
    let mut child = match Command::new("fastlane")
        .arg("fetch_locales")
        .current_dir(fastlane_dir)
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

fn read_fastlane_metadata_locales(metadata_path: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(metadata_path) else { return vec![] };
    let mut locales: Vec<String> = entries
        .flatten()
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "default" || name == "review_information" {
                return false;
            }
            e.file_type().map(|t| t.is_dir()).unwrap_or(false)
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    locales.sort();
    locales
}

/// Full fallback chain used during deploy: 1) fetch live from the store, 2) fastlane
/// metadata folders, 3) serconf.dart language flags.
pub async fn get_store_locales(workspace_path: &str) -> (Vec<String>, Vec<String>) {
    let root = Path::new(workspace_path);

    let ios_dir = root.join("ios");
    let android_dir = root.join("android");
    let (ios_result, android_result) =
        tokio::join!(run_fastlane_fetch_locales(&ios_dir), run_fastlane_fetch_locales(&android_dir));

    let ios_from_store = ios_result.unwrap_or_default();
    let android_from_store = android_result.unwrap_or_default();

    if !ios_from_store.is_empty() || !android_from_store.is_empty() {
        let ios = if !ios_from_store.is_empty() { ios_from_store } else { vec!["tr".to_string()] };
        let android = if !android_from_store.is_empty() { android_from_store } else { vec!["tr-TR".to_string()] };
        return (ios, android);
    }

    let ios_from_meta = read_fastlane_metadata_locales(&root.join("ios/fastlane/metadata"));
    let android_from_meta = read_fastlane_metadata_locales(&root.join("android/fastlane/metadata/android"));

    if !ios_from_meta.is_empty() || !android_from_meta.is_empty() {
        let ios = if !ios_from_meta.is_empty() { ios_from_meta } else { vec!["tr".to_string()] };
        let android = if !android_from_meta.is_empty() { android_from_meta } else { vec!["tr-TR".to_string()] };
        return (ios, android);
    }

    let mut ios = vec!["tr".to_string()];
    let mut android = vec!["tr-TR".to_string()];

    let Some(project_id) = std::fs::read_to_string(root.join("sermobileboss.txt")).ok().map(|s| s.trim().to_string()) else {
        return (ios, android);
    };
    if project_id.is_empty() {
        return (ios, android);
    }

    let serconf_path = root.join("lib/conf/sermobplus-projects").join(&project_id).join("serconf.dart");
    let Ok(content) = std::fs::read_to_string(&serconf_path) else { return (ios, android) };

    for (flag, ios_locale, android_locale) in FLAG_TO_LOCALES {
        let re = Regex::new(&format!(r"const\s+{flag}\s*=\s*true\s*;")).unwrap();
        if re.is_match(&content) {
            ios.push(ios_locale.to_string());
            android.push(android_locale.to_string());
        }
    }

    (ios, android)
}
