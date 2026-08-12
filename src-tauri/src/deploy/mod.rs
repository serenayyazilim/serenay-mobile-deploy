pub mod locales;
pub mod registry;
pub mod translate;

use regex::Regex;
use std::sync::LazyLock;

const TWO_FACTOR_PATTERNS: &[&str] = &[
    r"(?i)please enter the 6.?digit code",
    r"(?i)enter the 6.?digit",
    r"(?i)two.factor authentication",
    r"(?i)2.step verification",
    r"(?i)enter your 2 factor",
    r"(?i)verification code",
    r"(?i)please enter.*code",
    r"(?i)enter.*6.*digit",
    r"(?i)sms code",
    r"(?i)authenticator code",
];

static TWO_FACTOR_REGEXES: LazyLock<Vec<Regex>> =
    LazyLock::new(|| TWO_FACTOR_PATTERNS.iter().map(|p| Regex::new(p).unwrap()).collect());

pub fn is_two_factor_prompt(line: &str) -> bool {
    TWO_FACTOR_REGEXES.iter().any(|re| re.is_match(line))
}

/// `scripts/deploy.rb` ve `scripts/sync_versions.rb`'nin Tauri bundle'ında bulunacağı yol.
/// Dev modunda proje kökündeki `scripts/`, paketlenmiş üründe resource dizini kullanılır.
pub fn find_script_path(app: &tauri::AppHandle, script_name: &str) -> Option<std::path::PathBuf> {
    use tauri::Manager;

    if let Ok(resource_dir) = app.path().resource_dir() {
        let p = resource_dir.join("scripts").join(script_name);
        if p.exists() {
            return Some(p);
        }
    }

    // Dev fallback: src-tauri/scripts/
    let dev_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts").join(script_name);
    if dev_path.exists() {
        return Some(dev_path);
    }

    None
}
