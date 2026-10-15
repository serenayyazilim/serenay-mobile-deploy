pub mod locales;
pub mod registry;
pub mod translate;

use regex::Regex;
use std::sync::LazyLock;

// Only the actual code prompts: descriptive mentions of 2FA (fastlane's release notes,
// spaceship's "Two-factor Authentication is enabled" intro) must not open the code dialog.
const TWO_FACTOR_PATTERNS: &[&str] = &[
    r"(?i)please enter the 6.?digit code",
    r"(?i)enter the 6.?digit",
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

/// Path where `scripts/deploy.rb` and `scripts/sync_versions.rb` live in the Tauri bundle.
/// Uses the project root's `scripts/` in dev mode, and the resource directory in the packaged app.
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

#[cfg(test)]
mod tests {
    use super::is_two_factor_prompt;

    #[test]
    fn matches_only_code_prompts() {
        assert!(is_two_factor_prompt("Please enter the 6 digit code you received at +90 ••• •• 12:"));
        assert!(is_two_factor_prompt("Please enter the 6 digit code:"));
        assert!(!is_two_factor_prompt("* [spaceship] Show Apple's reason when it cannot start two-factor authentication (#30298)"));
        assert!(!is_two_factor_prompt("Two-factor Authentication (6 digits code) is enabled for account 'a@b.com'"));
    }
}
