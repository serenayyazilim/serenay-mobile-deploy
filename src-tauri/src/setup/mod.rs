//! Prepares a mobile project for this app's deploy flow: detects what kind of project it
//! is, generates the fastlane setup the deploy expects (or reports what an existing one
//! lacks) and checks what's still missing before the first deploy.

use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectKind {
    Flutter,
    Expo,
    ReactNative,
    NativeIos,
    NativeAndroid,
}

impl ProjectKind {
    pub fn label(self) -> &'static str {
        match self {
            ProjectKind::Flutter => "Flutter",
            ProjectKind::Expo => "Expo",
            ProjectKind::ReactNative => "React Native",
            ProjectKind::NativeIos => "iOS",
            ProjectKind::NativeAndroid => "Android",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Platform {
    Ios,
    Android,
}

fn package_json(root: &Path) -> Option<Value> {
    serde_json::from_str(&std::fs::read_to_string(root.join("package.json")).ok()?).ok()
}

fn has_dependency(pkg: &Value, name: &str) -> bool {
    ["dependencies", "devDependencies"].iter().any(|k| pkg.get(k).and_then(|d| d.get(name)).is_some())
}

/// First `*.<ext>` entry in `dir` (e.g. `Runner.xcodeproj`), ignoring CocoaPods' own project.
fn find_xcode_file(dir: &Path, ext: &str) -> Option<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == ext) && p.file_stem().is_some_and(|s| s != "Pods"))
        .collect();
    found.sort();
    found.into_iter().next()
}

/// Order matters: an Expo project is also a React Native one, and one that already has
/// native folders has been prebuilt, so it deploys like any React Native project.
pub fn detect_kind(root: &Path) -> Option<ProjectKind> {
    if root.join("pubspec.yaml").exists() {
        return Some(ProjectKind::Flutter);
    }
    if let Some(pkg) = package_json(root) {
        let prebuilt = root.join("ios").is_dir() && root.join("android").is_dir();
        if has_dependency(&pkg, "expo") && !prebuilt {
            return Some(ProjectKind::Expo);
        }
        if has_dependency(&pkg, "react-native") {
            return Some(ProjectKind::ReactNative);
        }
    }
    if root.join("gradlew").exists() && (root.join("settings.gradle").exists() || root.join("settings.gradle.kts").exists()) {
        return Some(ProjectKind::NativeAndroid);
    }
    if find_xcode_file(root, "xcworkspace").is_some() || find_xcode_file(root, "xcodeproj").is_some() {
        return Some(ProjectKind::NativeIos);
    }
    None
}

/// The (iOS, Android) folders that hold each platform's `fastlane/` folder.
pub fn platform_dirs(root: &Path, kind: ProjectKind) -> (Option<PathBuf>, Option<PathBuf>) {
    match kind {
        ProjectKind::Flutter | ProjectKind::ReactNative => (Some(root.join("ios")), Some(root.join("android"))),
        ProjectKind::NativeIos => (Some(root.to_path_buf()), None),
        ProjectKind::NativeAndroid => (None, Some(root.to_path_buf())),
        ProjectKind::Expo => (None, None),
    }
}

pub fn project_name(root: &Path, kind: ProjectKind) -> String {
    let name = match kind {
        ProjectKind::Flutter => std::fs::read_to_string(root.join("pubspec.yaml")).ok().and_then(|c| {
            Regex::new(r"(?m)^name:\s*(.+)$").unwrap().captures(&c).map(|c| c[1].trim().to_string())
        }),
        ProjectKind::Expo | ProjectKind::ReactNative => {
            package_json(root).and_then(|p| p.get("name").and_then(|n| n.as_str()).map(String::from))
        }
        ProjectKind::NativeIos | ProjectKind::NativeAndroid => None,
    };
    name.or_else(|| root.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "Mobile Project".to_string())
}

fn pbxproj_path(ios_dir: &Path) -> Option<PathBuf> {
    Some(find_xcode_file(ios_dir, "xcodeproj")?.join("project.pbxproj"))
}

pub fn read_ios_bundle_id(ios_dir: &Path) -> Option<String> {
    let content = std::fs::read_to_string(pbxproj_path(ios_dir)?).ok()?;
    // ponytail: the app's ID is taken as the shortest literal one, since test and extension
    // targets append to it; a project whose app ID is longer than an extension's would need
    // the target's build settings resolved instead.
    Regex::new(r"PRODUCT_BUNDLE_IDENTIFIER = ([^;]+);")
        .unwrap()
        .captures_iter(&content)
        .map(|c| c[1].trim().trim_matches('"').to_string())
        .filter(|id| !id.contains("$("))
        .min_by_key(|id| id.len())
}

pub fn read_android_package(android_dir: &Path) -> Option<String> {
    let re = Regex::new(r#"applicationId\s*=?\s*["']([^"']+)["']"#).unwrap();
    ["app/build.gradle.kts", "app/build.gradle"]
        .iter()
        .find_map(|f| re.captures(&std::fs::read_to_string(android_dir.join(f)).ok()?).map(|c| c[1].to_string()))
        .or_else(|| {
            let manifest = std::fs::read_to_string(android_dir.join("app/src/main/AndroidManifest.xml")).ok()?;
            Regex::new(r#"package="([^"]+)""#).unwrap().captures(&manifest).map(|c| c[1].to_string())
        })
}

pub fn read_team_id(ios_dir: &Path) -> Option<String> {
    let content = std::fs::read_to_string(pbxproj_path(ios_dir)?).ok()?;
    Regex::new(r"DEVELOPMENT_TEAM = ([A-Z0-9]+);").unwrap().captures(&content).map(|c| c[1].to_string())
}

/// Sets the team on every build configuration, adding it next to each bundle ID when
/// the project has none yet (automatic signing can't pick a team on its own).
pub fn write_team_id(ios_dir: &Path, team_id: &str) -> Result<(), String> {
    if !Regex::new(r"^[A-Z0-9]{10}$").unwrap().is_match(team_id) {
        return Err("A Team ID is 10 uppercase letters and digits".to_string());
    }
    let path = pbxproj_path(ios_dir).ok_or("Xcode project not found")?;
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    let existing = Regex::new(r"DEVELOPMENT_TEAM = [^;]*;").unwrap();
    let updated = if existing.is_match(&content) {
        existing.replace_all(&content, format!("DEVELOPMENT_TEAM = {team_id};"))
    } else {
        Regex::new(r"(?m)^([ \t]*)(PRODUCT_BUNDLE_IDENTIFIER = [^;]+;)$")
            .unwrap()
            .replace_all(&content, format!("${{1}}DEVELOPMENT_TEAM = {team_id};\n${{1}}${{2}}"))
    };
    std::fs::write(&path, updated.as_ref()).map_err(|e| e.to_string())
}

/// Google Play rejects both unsigned bundles and ones signed with the debug key, which is
/// what the Flutter and React Native templates use for release builds.
pub fn android_release_signing_ok(android_dir: &Path) -> bool {
    let Some(content) = ["app/build.gradle.kts", "app/build.gradle"]
        .iter()
        .find_map(|f| std::fs::read_to_string(android_dir.join(f)).ok())
    else {
        return false;
    };
    let Some(build_types) = content.find("buildTypes").map(|i| &content[i..]) else { return false };
    let Some(release) = Regex::new(r"release\s*\{([^}]*)\}").unwrap().captures(build_types) else { return false };
    let block = &release[1];
    block.contains("signingConfig") && !Regex::new(r#"signingConfigs\.(debug|getByName\(\s*"debug"\s*\))"#).unwrap().is_match(block)
}

// ============ VERSION ============
//
// Flutter keeps `x.y.z+build` in pubspec.yaml and deploy.rb bumps it. The other kinds keep
// it in the native projects (and app.json for Expo); these read and write those places.

fn gradle_path(android_dir: &Path) -> Option<PathBuf> {
    ["app/build.gradle.kts", "app/build.gradle"].iter().map(|f| android_dir.join(f)).find(|p| p.exists())
}

pub fn is_expo(root: &Path) -> bool {
    package_json(root).is_some_and(|p| has_dependency(&p, "expo"))
}

/// Current version as `x.y.z+build`, from Android first, then iOS.
pub fn read_version(root: &Path, kind: ProjectKind) -> Option<String> {
    if kind == ProjectKind::Flutter {
        let pubspec = std::fs::read_to_string(root.join("pubspec.yaml")).ok()?;
        return Regex::new(r"(?m)^version:\s*(.+)$").unwrap().captures(&pubspec).map(|c| c[1].trim().to_string());
    }
    let (ios_dir, android_dir) = platform_dirs(root, kind);
    let from_android = android_dir.and_then(|dir| {
        let gradle = std::fs::read_to_string(gradle_path(&dir)?).ok()?;
        let name = Regex::new(r#"versionName\s*=?\s*"([^"]+)""#).unwrap().captures(&gradle)?[1].to_string();
        let code = Regex::new(r"versionCode\s*=?\s*(\d+)").unwrap().captures(&gradle)?[1].to_string();
        Some(format!("{name}+{code}"))
    });
    from_android.or_else(|| {
        let pbxproj = std::fs::read_to_string(pbxproj_path(&ios_dir?)?).ok()?;
        let name = Regex::new(r"MARKETING_VERSION = ([^;]+);").unwrap().captures(&pbxproj)?[1].trim().to_string();
        let build = Regex::new(r"CURRENT_PROJECT_VERSION = ([^;]+);").unwrap().captures(&pbxproj)?[1].trim().to_string();
        Some(format!("{name}+{build}"))
    })
}

/// Same scheme as deploy.rb's `auto_increment_version`: bump the patch with a carry at 9,
/// build number = major·10⁷ + minor·10⁵ + patch·10³.
pub fn next_version(current: &str) -> String {
    let mut parts = current.split('+').next().unwrap_or("").split('.').map(|p| p.trim().parse::<u64>().unwrap_or(0));
    let (mut major, mut minor, mut patch) = (parts.next().unwrap_or(1), parts.next().unwrap_or(0), parts.next().unwrap_or(0) + 1);
    if patch > 9 {
        patch = 0;
        minor += 1;
    }
    if minor > 9 {
        minor = 0;
        major += 1;
    }
    format!("{major}.{minor}.{patch}+{}", major * 10_000_000 + minor * 100_000 + patch * 1_000)
}

fn replace_in_file(path: &Path, replacements: &[(&str, String)]) -> Result<(), String> {
    let mut content = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    for (pattern, replacement) in replacements {
        content = Regex::new(pattern).unwrap().replace_all(&content, replacement.as_str()).to_string();
    }
    std::fs::write(path, content).map_err(|e| e.to_string())
}

/// Writes `x.y.z+build` to every place the kind builds from.
// ponytail: an Info.plist with a hardcoded CFBundleShortVersionString (instead of
// $(MARKETING_VERSION)) isn't updated; patch it too if an older project needs that.
pub fn write_version(root: &Path, kind: ProjectKind, version: &str) -> Result<(), String> {
    let (name, build) = version.split_once('+').ok_or("Version must look like 1.2.3+10203000")?;
    let (ios_dir, android_dir) = platform_dirs(root, kind);

    if let Some(path) = android_dir.as_deref().and_then(gradle_path) {
        replace_in_file(&path, &[
            (r#"(versionName\s*=?\s*)"[^"]*""#, format!("${{1}}\"{name}\"")),
            (r"(versionCode\s*=?\s*)\d+", format!("${{1}}{build}")),
        ])?;
    }
    if let Some(path) = ios_dir.as_deref().and_then(pbxproj_path) {
        replace_in_file(&path, &[
            (r"MARKETING_VERSION = [^;]+;", format!("MARKETING_VERSION = {name};")),
            (r"CURRENT_PROJECT_VERSION = [^;]+;", format!("CURRENT_PROJECT_VERSION = {build};")),
        ])?;
    }
    if is_expo(root) {
        write_expo_version(root, name, build)?;
    }
    Ok(())
}

/// A later `expo prebuild` regenerates the native projects from app.json, so the version
/// goes there too. An app.config.* may override it, which only Expo itself can tell.
fn write_expo_version(root: &Path, name: &str, build: &str) -> Result<(), String> {
    let Some(mut app_json) = read_app_json(root) else { return Ok(()) };
    let expo = expo_section(&mut app_json);
    expo["version"] = Value::String(name.to_string());
    for (platform, key, value) in [("ios", "buildNumber", Value::String(build.to_string())), ("android", "versionCode", Value::from(build.parse::<u64>().unwrap_or(0)))] {
        if !expo.get(platform).is_some_and(|p| p.is_object()) {
            expo[platform] = serde_json::json!({});
        }
        expo[platform][key] = value;
    }
    let json = serde_json::to_string_pretty(&app_json).map_err(|e| e.to_string())?;
    std::fs::write(root.join("app.json"), format!("{json}\n")).map_err(|e| e.to_string())?;

    if !EXPO_DYNAMIC_CONFIGS.iter().any(|f| root.join(f).exists()) {
        return Ok(());
    }
    let output = std::process::Command::new("npx")
        .args(["expo", "config", "--json", "--type", "public"])
        .current_dir(root)
        .env("CI", "1")
        .output()
        .map_err(|e| format!("Could not run npx expo config: {e}"))?;
    let resolved: Value = serde_json::from_slice(&output.stdout).map_err(|_| "Could not read the resolved Expo config".to_string())?;
    if resolved.get("version").and_then(|v| v.as_str()) != Some(name) {
        return Err(format!(
            "The version was written to app.json, but app.config.* overrides it. Make app.config.* take \"version\", \"ios.buildNumber\" and \"android.versionCode\" from app.json (spread ...config)."
        ));
    }
    Ok(())
}

// ============ FASTLANE ============

const IOS_FASTFILE: &str = include_str!("templates/ios_Fastfile.rb");
const IOS_APPFILE: &str = include_str!("templates/ios_Appfile.rb");
const ANDROID_FASTFILE: &str = include_str!("templates/android_Fastfile.rb");
const ANDROID_APPFILE: &str = include_str!("templates/android_Appfile.rb");
const CONTRACT_LANES: [&str; 3] = ["beta", "release", "fetch_locales"];
pub const PLAY_KEY_FILE: &str = "play-store-key.json";

fn js_install(root: &Path) -> &'static str {
    if root.join("yarn.lock").exists() {
        "yarn install"
    } else if root.join("pnpm-lock.yaml").exists() {
        "pnpm install"
    } else if root.join("bun.lockb").exists() || root.join("bun.lock").exists() {
        "bun install"
    } else {
        "npm install"
    }
}

/// `build_app` arguments: Flutter always ships `Runner`; otherwise the Xcode project's
/// name, preferring a shared scheme of that name (what React Native and Xcode create).
fn ios_build_args(ios_dir: &Path, kind: ProjectKind) -> String {
    if kind == ProjectKind::Flutter {
        return r#"workspace: "Runner.xcworkspace", scheme: "Runner""#.to_string();
    }
    let project = find_xcode_file(ios_dir, "xcodeproj");
    let name = project
        .as_ref()
        .and_then(|p| p.file_stem())
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "App".to_string());

    let schemes: Vec<String> = project
        .as_ref()
        .and_then(|p| std::fs::read_dir(p.join("xcshareddata/xcschemes")).ok())
        .map(|entries| entries.flatten().filter_map(|e| e.path().file_stem().map(|s| s.to_string_lossy().to_string())).collect())
        .unwrap_or_default();
    let scheme = if schemes.contains(&name) { name.clone() } else { schemes.into_iter().min().unwrap_or_else(|| name.clone()) };

    if ios_dir.join("Podfile").exists() {
        format!(r#"workspace: "{name}.xcworkspace", scheme: "{scheme}""#)
    } else {
        format!(r#"project: "{name}.xcodeproj", scheme: "{scheme}""#)
    }
}

/// Ruby lines that build the app. `sh` runs from the `fastlane/` folder while actions
/// such as `gradle` run from its parent — that's fastlane's own convention. Flutter gets
/// none: deploy.rb already runs pub get, pod install and the app bundle build for it.
fn prepare_lines(root: &Path, platform_dir: &Path, kind: ProjectKind, platform: Platform) -> String {
    let to_root = if platform_dir == root { ".." } else { "../.." };
    let lines: Vec<String> = match (kind, platform) {
        (ProjectKind::ReactNative, Platform::Ios) => {
            vec![format!("sh(\"cd {to_root} && {}\")", js_install(root)), "sh(\"cd .. && pod install\")".into()]
        }
        (ProjectKind::ReactNative, Platform::Android) => vec![
            format!("sh(\"cd {to_root} && {}\")", js_install(root)),
            "gradle(task: \"bundle\", build_type: \"Release\")".into(),
        ],
        (ProjectKind::NativeIos, _) if platform_dir.join("Podfile").exists() => vec!["sh(\"cd .. && pod install\")".into()],
        (ProjectKind::NativeAndroid, _) => vec!["gradle(task: \"bundle\", build_type: \"Release\")".into()],
        _ => vec![],
    };
    lines.iter().map(|l| format!("    {l}")).collect::<Vec<_>>().join("\n")
}

/// A private lane the contract lanes rely on: (name, snippet).
fn helper_lanes(platform: Platform) -> &'static [(&'static str, &'static str)] {
    match platform {
        Platform::Ios => &[
            ("sermobile_api_key", include_str!("templates/ios_api_key.rb")),
            ("build_ipa", include_str!("templates/ios_build.rb")),
        ],
        Platform::Android => &[("build_aab", include_str!("templates/android_build.rb"))],
    }
}

fn contract_lane(platform: Platform, lane: &str) -> &'static str {
    match (platform, lane) {
        (Platform::Ios, "beta") => include_str!("templates/ios_beta.rb"),
        (Platform::Ios, "release") => include_str!("templates/ios_release.rb"),
        (Platform::Ios, _) => include_str!("templates/ios_fetch_locales.rb"),
        (Platform::Android, "beta") => include_str!("templates/android_beta.rb"),
        (Platform::Android, "release") => include_str!("templates/android_release.rb"),
        (Platform::Android, _) => include_str!("templates/android_fetch_locales.rb"),
    }
}

fn has_lane(fastfile: &str, lane: &str) -> bool {
    Regex::new(&format!(r"lane\s+:{lane}\b")).unwrap().is_match(fastfile)
}

/// The given contract lanes plus the helper lanes they need that `fastfile` doesn't
/// define yet, with the project's build steps filled in.
fn lane_snippets(root: &Path, platform_dir: &Path, kind: ProjectKind, platform: Platform, lanes: &[&str], fastfile: &str) -> String {
    let helpers = helper_lanes(platform).iter().filter(|(name, _)| !has_lane(fastfile, name)).map(|(_, snippet)| *snippet);
    let aab = if kind == ProjectKind::Flutter {
        // Flutter builds outside Gradle, so supply can't pick the bundle up from Gradle's output.
        "\n      aab: \"../build/app/outputs/bundle/release/app-release.aab\","
    } else {
        ""
    };
    helpers
        .chain(lanes.iter().map(|lane| contract_lane(platform, lane)))
        .collect::<Vec<_>>()
        .join("\n")
        .replace("{{PREPARE}}", &prepare_lines(root, platform_dir, kind, platform))
        .replace("{{BUILD_ARGS}}", &ios_build_args(platform_dir, kind))
        .replace("{{AAB_ARG}}", aab)
}

fn appfile_entries(platform_dir: &Path, platform: Platform) -> Vec<(&'static str, String)> {
    match platform {
        Platform::Ios => vec![("app_identifier", IOS_APPFILE.replace("{{APP_IDENTIFIER}}", &read_ios_bundle_id(platform_dir).unwrap_or_default()))],
        Platform::Android => {
            let appfile = ANDROID_APPFILE.replace("{{PACKAGE_NAME}}", &read_android_package(platform_dir).unwrap_or_default());
            let mut lines = appfile.lines().map(|l| format!("{l}\n"));
            vec![("package_name", lines.next().unwrap_or_default()), ("json_key_file", lines.next().unwrap_or_default())]
        }
    }
}

fn render(root: &Path, platform_dir: &Path, kind: ProjectKind, platform: Platform) -> (String, String) {
    let template = match platform {
        Platform::Ios => IOS_FASTFILE,
        Platform::Android => ANDROID_FASTFILE,
    };
    let lanes = lane_snippets(root, platform_dir, kind, platform, &CONTRACT_LANES, "");
    let appfile = appfile_entries(platform_dir, platform).into_iter().map(|(_, line)| line).collect();
    (template.replace("{{LANES}}", lanes.trim_end()), appfile)
}

fn ruby_syntax_ok(path: &Path) -> bool {
    std::process::Command::new("ruby").arg("-c").arg(path).output().is_ok_and(|o| o.status.success())
}

/// Adds what an existing fastlane setup lacks without touching what's there: missing lanes
/// go just before the closing `end` of its `platform` block (or at the end of the file),
/// missing Appfile settings are appended. A Fastfile that no longer parses is restored.
pub fn complete_fastlane(root: &Path, kind: ProjectKind) -> Result<(), String> {
    for (platform, dir) in platforms(root, kind) {
        let fastlane = dir.join("fastlane");
        if !fastlane.exists() {
            continue;
        }

        let fastfile_path = fastlane.join("Fastfile");
        let original = std::fs::read_to_string(&fastfile_path).unwrap_or_default();
        let missing: Vec<&str> = CONTRACT_LANES.into_iter().filter(|lane| !has_lane(&original, lane)).collect();
        if !missing.is_empty() {
            let snippets = lane_snippets(root, &dir, kind, platform, &missing, &original);
            let block = format!("\n  # Added by Serenay Mobile Deploy\n{}", snippets.trim_end());
            // ponytail: the platform block is taken to close with the last unindented `end`;
            // a Fastfile laid out differently gets the lanes at its end, outside the block.
            let closing = Regex::new(r"(?m)^end\s*$").unwrap().find_iter(&original).last().filter(|_| original.contains("platform :"));
            let updated = match closing {
                Some(m) => format!("{}{block}\n{}", &original[..m.start()], &original[m.start()..]),
                None => format!("{}\n{block}\n", original.trim_end()),
            };
            std::fs::write(&fastfile_path, &updated).map_err(|e| e.to_string())?;
            if !ruby_syntax_ok(&fastfile_path) {
                std::fs::write(&fastfile_path, &original).map_err(|e| e.to_string())?;
                return Err(format!("Could not add the lanes to {} automatically; the file was left unchanged", fastfile_path.display()));
            }
        }

        let appfile_path = fastlane.join("Appfile");
        let mut appfile = std::fs::read_to_string(&appfile_path).unwrap_or_default();
        let before = appfile.clone();
        for (key, line) in appfile_entries(&dir, platform) {
            let empty = format!("{key}(\"\")");
            if appfile.contains(&empty) {
                appfile = appfile.replace(&format!("{empty}\n"), &line);
            } else if !appfile.contains(key) {
                if !appfile.is_empty() && !appfile.ends_with('\n') {
                    appfile.push('\n');
                }
                appfile.push_str(&line);
            }
        }
        if appfile != before {
            std::fs::write(&appfile_path, appfile).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn platforms(root: &Path, kind: ProjectKind) -> Vec<(Platform, PathBuf)> {
    let (ios, android) = platform_dirs(root, kind);
    ios.map(|d| (Platform::Ios, d)).into_iter().chain(android.map(|d| (Platform::Android, d))).collect()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Applied {
    pub platform: Platform,
    pub generated: bool,
}

const GITIGNORE_ENTRIES: &[&str] = &[
    "sermobileboss_appstoreconnect.json",
    "**/fastlane/play-store-key.json",
    "**/fastlane/report.xml",
    "*.ipa",
    "*.dSYM.zip",
];

fn ensure_gitignore(root: &Path) -> std::io::Result<()> {
    let path = root.join(".gitignore");
    let mut content = std::fs::read_to_string(&path).unwrap_or_default();
    let missing: Vec<&str> = GITIGNORE_ENTRIES.iter().copied().filter(|e| !content.lines().any(|l| l.trim() == *e)).collect();
    if missing.is_empty() {
        return Ok(());
    }
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&format!("\n# Serenay Mobile Deploy\n{}\n", missing.join("\n")));
    std::fs::write(path, content)
}

/// Generates fastlane for each platform that has none. An existing `fastlane/` folder is
/// never touched — `readiness` reports what it lacks instead.
pub fn apply(root: &Path, kind: ProjectKind) -> Result<Vec<Applied>, String> {
    ensure_gitignore(root).map_err(|e| e.to_string())?;
    platforms(root, kind)
        .into_iter()
        .map(|(platform, dir)| {
            let fastlane = dir.join("fastlane");
            if fastlane.exists() {
                return Ok(Applied { platform, generated: false });
            }
            let (fastfile, appfile) = render(root, &dir, kind, platform);
            std::fs::create_dir_all(&fastlane).map_err(|e| e.to_string())?;
            std::fs::write(fastlane.join("Fastfile"), fastfile).map_err(|e| e.to_string())?;
            std::fs::write(fastlane.join("Appfile"), appfile).map_err(|e| e.to_string())?;
            Ok(Applied { platform, generated: true })
        })
        .collect()
}

/// What a `fastlane/` folder lacks for this app's deploy contract.
fn missing_in_fastlane(fastlane: &Path, platform: Platform) -> Vec<String> {
    let fastfile = std::fs::read_to_string(fastlane.join("Fastfile")).unwrap_or_default();
    let appfile = std::fs::read_to_string(fastlane.join("Appfile")).unwrap_or_default();

    let lanes = CONTRACT_LANES
        .into_iter()
        .filter(|lane| !has_lane(&fastfile, lane))
        .map(|lane| format!("Fastfile: lane :{lane}"));
    let keys: &[&str] = match platform {
        Platform::Ios => &["app_identifier"],
        Platform::Android => &["package_name", "json_key_file"],
    };
    let settings = keys
        .iter()
        .filter(|key| !appfile.contains(*key) || appfile.contains(&format!("{key}(\"\")")))
        .map(|key| format!("Appfile: {key}"));
    lanes.chain(settings).collect()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IosReadiness {
    pub fastlane_missing: Vec<String>,
    pub team_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AndroidReadiness {
    pub fastlane_missing: Vec<String>,
    /// `None` when the Appfile points at a key this app didn't place; only the store check can tell then.
    pub play_key: Option<bool>,
    pub release_signing: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Readiness {
    pub kind: ProjectKind,
    pub ios: Option<IosReadiness>,
    pub android: Option<AndroidReadiness>,
}

/// Local checks only, so it answers instantly; the store records are checked separately.
pub fn readiness(root: &Path, kind: ProjectKind) -> Readiness {
    let (ios_dir, android_dir) = platform_dirs(root, kind);
    Readiness {
        kind,
        ios: ios_dir.map(|dir| IosReadiness {
            fastlane_missing: missing_in_fastlane(&dir.join("fastlane"), Platform::Ios),
            team_id: read_team_id(&dir),
        }),
        android: android_dir.map(|dir| {
            let appfile = std::fs::read_to_string(dir.join("fastlane/Appfile")).unwrap_or_default();
            AndroidReadiness {
                fastlane_missing: missing_in_fastlane(&dir.join("fastlane"), Platform::Android),
                play_key: if appfile.contains(PLAY_KEY_FILE) {
                    Some(dir.join("fastlane").join(PLAY_KEY_FILE).exists())
                } else if appfile.contains("json_key_file") {
                    None
                } else {
                    Some(false)
                },
                release_signing: android_release_signing_ok(&dir),
            }
        }),
    }
}

/// Copies a Google Play service account key to where the generated Appfile expects it.
pub fn set_play_key(root: &Path, kind: ProjectKind, source: &Path) -> Result<(), String> {
    let (_, android_dir) = platform_dirs(root, kind);
    let android_dir = android_dir.ok_or("Not an Android project")?;
    let key: Value = serde_json::from_str(&std::fs::read_to_string(source).map_err(|e| e.to_string())?)
        .map_err(|_| "The file is not valid JSON".to_string())?;
    if key.get("type").and_then(|t| t.as_str()) != Some("service_account") {
        return Err("This is not a Google service account key (\"type\": \"service_account\" is missing)".to_string());
    }
    let fastlane = android_dir.join("fastlane");
    std::fs::create_dir_all(&fastlane).map_err(|e| e.to_string())?;
    std::fs::copy(source, fastlane.join(PLAY_KEY_FILE)).map(|_| ()).map_err(|e| e.to_string())
}

/// Where each kind's Android build leaves its release bundle.
pub fn aab_path(root: &Path, kind: ProjectKind) -> Option<PathBuf> {
    let (_, android_dir) = platform_dirs(root, kind);
    let base = if kind == ProjectKind::Flutter { root.join("build") } else { android_dir?.join("app/build") };
    Some(base.join("outputs/bundle/release/app-release.aab"))
}

/// Shell command that builds the release bundle, run from the project root.
pub fn aab_build_command(kind: ProjectKind) -> Option<&'static str> {
    match kind {
        ProjectKind::Flutter => Some("flutter build appbundle"),
        ProjectKind::ReactNative => Some("cd android && ./gradlew bundleRelease"),
        ProjectKind::NativeAndroid => Some("./gradlew bundleRelease"),
        ProjectKind::Expo | ProjectKind::NativeIos => None,
    }
}

// ============ EXPO ============

const EXPO_DYNAMIC_CONFIGS: &[&str] = &["app.config.js", "app.config.ts", "app.config.mjs", "app.config.cjs"];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpoConfig {
    /// `app.config.*` is code, so this app can't safely edit it.
    pub dynamic: bool,
    pub bundle_identifier: Option<String>,
    pub package: Option<String>,
}

fn read_app_json(root: &Path) -> Option<Value> {
    serde_json::from_str(&std::fs::read_to_string(root.join("app.json")).ok()?).ok()
}

/// app.json keeps its settings under `expo`, or at the root in older projects.
fn expo_section(app_json: &mut Value) -> &mut Value {
    if app_json.get("expo").is_some() { &mut app_json["expo"] } else { app_json }
}

pub fn read_expo_config(root: &Path) -> ExpoConfig {
    let mut app_json = read_app_json(root).unwrap_or(Value::Null);
    let expo = expo_section(&mut app_json);
    let get = |platform: &str, key: &str| expo.get(platform).and_then(|p| p.get(key)).and_then(|v| v.as_str()).map(String::from);
    ExpoConfig {
        dynamic: EXPO_DYNAMIC_CONFIGS.iter().any(|f| root.join(f).exists()),
        bundle_identifier: get("ios", "bundleIdentifier"),
        package: get("android", "package"),
    }
}

/// Prebuild fails non-interactively without these, so the confirm modal asks for them first.
pub fn write_expo_ids(root: &Path, bundle_identifier: Option<&str>, package: Option<&str>) -> Result<(), String> {
    if bundle_identifier.is_none() && package.is_none() {
        return Ok(());
    }
    let id = Regex::new(r"^[A-Za-z][A-Za-z0-9_]*(\.[A-Za-z][A-Za-z0-9_-]*)+$").unwrap();
    for value in [bundle_identifier, package].into_iter().flatten() {
        if !id.is_match(value) {
            return Err(format!("Invalid identifier: {value}"));
        }
    }
    let mut app_json = read_app_json(root).ok_or("app.json not found")?;
    let expo = expo_section(&mut app_json);
    for (platform, key, value) in [("ios", "bundleIdentifier", bundle_identifier), ("android", "package", package)] {
        if let Some(value) = value {
            if !expo.get(platform).is_some_and(|p| p.is_object()) {
                expo[platform] = serde_json::json!({});
            }
            expo[platform][key] = Value::String(value.to_string());
        }
    }
    let json = serde_json::to_string_pretty(&app_json).map_err(|e| e.to_string())?;
    std::fs::write(root.join("app.json"), format!("{json}\n")).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(files: &[(&str, &str)]) -> PathBuf {
        let root = std::env::temp_dir().join(format!("setup-test-{}", uuid::Uuid::new_v4()));
        for (path, content) in files {
            let path = root.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        }
        root
    }

    const FLUTTER_PBXPROJ: &str = "\t\t\t\tPRODUCT_BUNDLE_IDENTIFIER = com.acme.app.RunnerTests;\n\t\t\t\tPRODUCT_BUNDLE_IDENTIFIER = com.acme.app;\n";

    #[test]
    fn detects_each_kind() {
        let expo = r#"{"dependencies":{"expo":"~51.0.0","react-native":"0.74.0"}}"#;
        let cases = [
            (project(&[("pubspec.yaml", "name: app")]), Some(ProjectKind::Flutter)),
            (project(&[("package.json", expo)]), Some(ProjectKind::Expo)),
            (project(&[("package.json", expo), ("ios/Podfile", ""), ("android/build.gradle", "")]), Some(ProjectKind::ReactNative)),
            (project(&[("package.json", r#"{"dependencies":{"react-native":"0.76.0"}}"#)]), Some(ProjectKind::ReactNative)),
            (project(&[("gradlew", ""), ("settings.gradle.kts", "")]), Some(ProjectKind::NativeAndroid)),
            (project(&[("Shop.xcodeproj/project.pbxproj", "")]), Some(ProjectKind::NativeIos)),
            (project(&[("README.md", "")]), None),
        ];
        for (root, kind) in cases {
            assert_eq!(detect_kind(&root), kind, "{}", root.display());
        }
    }

    #[test]
    fn reads_app_bundle_id_over_test_target() {
        let root = project(&[("ios/Runner.xcodeproj/project.pbxproj", FLUTTER_PBXPROJ)]);
        assert_eq!(read_ios_bundle_id(&root.join("ios")).as_deref(), Some("com.acme.app"));
    }

    #[test]
    fn writes_team_id_next_to_each_bundle_id() {
        let root = project(&[("ios/Runner.xcodeproj/project.pbxproj", FLUTTER_PBXPROJ)]);
        let ios = root.join("ios");
        assert!(write_team_id(&ios, "abc").is_err());
        write_team_id(&ios, "ABCDE12345").unwrap();
        let content = std::fs::read_to_string(pbxproj_path(&ios).unwrap()).unwrap();
        assert_eq!(content.matches("\t\t\t\tDEVELOPMENT_TEAM = ABCDE12345;\n").count(), 2);
        write_team_id(&ios, "ZZZZZ99999").unwrap();
        assert_eq!(read_team_id(&ios).as_deref(), Some("ZZZZZ99999"));
    }

    #[test]
    fn debug_or_missing_release_signing_is_not_ok() {
        let gradle = |release: &str| {
            let root = project(&[(
                "android/app/build.gradle",
                &format!("android {{\n signingConfigs {{ release {{ storeFile file('k') }} }}\n buildTypes {{\n debug {{ signingConfig signingConfigs.debug }}\n release {{ {release} }}\n }}\n}}"),
            )]);
            android_release_signing_ok(&root.join("android"))
        };
        assert!(!gradle("signingConfig signingConfigs.debug"));
        assert!(!gradle("signingConfig = signingConfigs.getByName(\"debug\")"));
        assert!(!gradle("minifyEnabled false"));
        assert!(gradle("signingConfig signingConfigs.release"));
    }

    #[test]
    fn generates_valid_fastlane_once_and_reports_existing() {
        let root = project(&[
            ("pubspec.yaml", "name: app"),
            ("ios/Runner.xcodeproj/project.pbxproj", FLUTTER_PBXPROJ),
            ("android/app/build.gradle.kts", "defaultConfig { applicationId = \"com.acme.app\" }"),
        ]);
        let applied = apply(&root, ProjectKind::Flutter).unwrap();
        assert!(applied.iter().all(|a| a.generated));

        for platform in ["ios", "android"] {
            let fastfile = root.join(platform).join("fastlane/Fastfile");
            let ok = std::process::Command::new("ruby").arg("-c").arg(&fastfile).output().unwrap().status.success();
            assert!(ok, "{platform} Fastfile is not valid Ruby");
        }
        assert!(std::fs::read_to_string(root.join("ios/fastlane/Appfile")).unwrap().contains("app_identifier(\"com.acme.app\")"));
        assert!(std::fs::read_to_string(root.join(".gitignore")).unwrap().contains("**/fastlane/play-store-key.json"));

        let ready = readiness(&root, ProjectKind::Flutter);
        assert!(ready.ios.unwrap().fastlane_missing.is_empty());
        assert_eq!(ready.android.as_ref().unwrap().play_key, Some(false));

        std::fs::write(root.join("android/fastlane/Fastfile"), "lane :beta do\nend").unwrap();
        assert!(!apply(&root, ProjectKind::Flutter).unwrap().iter().any(|a| a.generated));
        let missing = readiness(&root, ProjectKind::Flutter).android.unwrap().fastlane_missing;
        assert_eq!(missing, ["Fastfile: lane :release", "Fastfile: lane :fetch_locales"]);
    }

    #[test]
    fn generates_valid_fastlane_for_react_native_and_native() {
        let rn = project(&[
            ("package.json", r#"{"dependencies":{"react-native":"0.76.0"}}"#),
            ("yarn.lock", ""),
            ("ios/Podfile", ""),
            ("ios/Shop.xcodeproj/xcshareddata/xcschemes/Shop.xcscheme", ""),
            ("android/app/build.gradle", "applicationId \"com.acme.shop\""),
        ]);
        let ios_only = project(&[("Shop.xcodeproj/project.pbxproj", FLUTTER_PBXPROJ)]);
        let android_only = project(&[("gradlew", ""), ("settings.gradle", ""), ("app/build.gradle", "applicationId 'com.acme'")]);

        for (root, kind) in [(&rn, ProjectKind::ReactNative), (&ios_only, ProjectKind::NativeIos), (&android_only, ProjectKind::NativeAndroid)] {
            apply(root, kind).unwrap();
            for (_, dir) in platforms(root, kind) {
                let fastfile = dir.join("fastlane/Fastfile");
                let ok = std::process::Command::new("ruby").arg("-c").arg(&fastfile).output().unwrap().status.success();
                assert!(ok, "{} is not valid Ruby", fastfile.display());
            }
        }
        let rn_ios = std::fs::read_to_string(rn.join("ios/fastlane/Fastfile")).unwrap();
        assert!(rn_ios.contains(r#"workspace: "Shop.xcworkspace", scheme: "Shop""#) && rn_ios.contains("cd ../.. && yarn install"));
        assert!(std::fs::read_to_string(rn.join("android/fastlane/Fastfile")).unwrap().contains("gradle(task: \"bundle\""));
        assert!(std::fs::read_to_string(ios_only.join("fastlane/Fastfile")).unwrap().contains(r#"project: "Shop.xcodeproj", scheme: "Shop""#));
        assert!(std::fs::read_to_string(android_only.join("fastlane/Appfile")).unwrap().contains("package_name(\"com.acme\")"));
    }

    #[test]
    fn completes_existing_fastlane_without_touching_its_lanes() {
        let existing_ios = "default_platform(:ios)\n\nplatform :ios do\n  lane :beta do\n    puts \"mine\"\n  end\nend\n";
        let root = project(&[
            ("pubspec.yaml", "name: app"),
            ("ios/Runner.xcodeproj/project.pbxproj", FLUTTER_PBXPROJ),
            ("ios/fastlane/Fastfile", existing_ios),
            ("ios/fastlane/Appfile", "apple_id(\"me@example.com\")"),
            ("android/app/build.gradle", "applicationId \"com.acme.app\""),
            ("android/fastlane/Fastfile", "# nothing yet\n"),
            ("android/fastlane/Appfile", "package_name(\"\")\n"),
        ]);
        complete_fastlane(&root, ProjectKind::Flutter).unwrap();

        let ready = readiness(&root, ProjectKind::Flutter);
        assert!(ready.ios.unwrap().fastlane_missing.is_empty());
        assert!(ready.android.unwrap().fastlane_missing.is_empty());
        let ios = std::fs::read_to_string(root.join("ios/fastlane/Fastfile")).unwrap();
        assert!(ios.starts_with("default_platform(:ios)\n\nplatform :ios do\n  lane :beta do\n    puts \"mine\""));
        assert_eq!(ios.matches("lane :beta").count(), 1);
        assert!(ios.trim_end().ends_with("end") && ruby_syntax_ok(&root.join("ios/fastlane/Fastfile")));
        assert!(ruby_syntax_ok(&root.join("android/fastlane/Fastfile")));
        let appfile = std::fs::read_to_string(root.join("ios/fastlane/Appfile")).unwrap();
        assert!(appfile.starts_with("apple_id(\"me@example.com\")\napp_identifier(\"com.acme.app\")"));
        assert!(std::fs::read_to_string(root.join("android/fastlane/Appfile")).unwrap().starts_with("package_name(\"com.acme.app\")\njson_key_file("));
    }

    #[test]
    fn bumps_version_like_deploy_rb() {
        assert_eq!(next_version("1.0+1"), "1.0.1+10001000");
        assert_eq!(next_version("2.3.9+20309000"), "2.4.0+20400000");
        assert_eq!(next_version("1.9.9"), "2.0.0+20000000");
    }

    #[test]
    fn reads_and_writes_native_and_expo_versions() {
        let rn = project(&[
            ("package.json", r#"{"dependencies":{"expo":"~51.0.0","react-native":"0.74.0"}}"#),
            ("app.json", r#"{"expo":{"name":"Shop","version":"1.0.0"}}"#),
            ("ios/Shop.xcodeproj/project.pbxproj", "\t\t\t\tCURRENT_PROJECT_VERSION = 1;\n\t\t\t\tMARKETING_VERSION = 1.0;\n\t\t\t\tCURRENT_PROJECT_VERSION = 1;\n"),
            ("android/app/build.gradle", "defaultConfig {\n    versionCode 1\n    versionName \"1.0\"\n}"),
        ]);
        assert_eq!(read_version(&rn, ProjectKind::ReactNative).as_deref(), Some("1.0+1"));
        write_version(&rn, ProjectKind::ReactNative, "1.0.1+10001000").unwrap();
        assert_eq!(read_version(&rn, ProjectKind::ReactNative).as_deref(), Some("1.0.1+10001000"));
        let pbxproj = std::fs::read_to_string(rn.join("ios/Shop.xcodeproj/project.pbxproj")).unwrap();
        assert_eq!(pbxproj.matches("CURRENT_PROJECT_VERSION = 10001000;").count(), 2);
        let app_json: Value = serde_json::from_str(&std::fs::read_to_string(rn.join("app.json")).unwrap()).unwrap();
        assert_eq!(app_json["expo"]["version"], "1.0.1");
        assert_eq!(app_json["expo"]["ios"]["buildNumber"], "10001000");
        assert_eq!(app_json["expo"]["android"]["versionCode"], 10001000);

        let android = project(&[("gradlew", ""), ("settings.gradle.kts", ""), ("app/build.gradle.kts", "versionCode = 7\nversionName = \"3.1.4\"")]);
        assert_eq!(read_version(&android, ProjectKind::NativeAndroid).as_deref(), Some("3.1.4+7"));
        write_version(&android, ProjectKind::NativeAndroid, "3.1.5+30105000").unwrap();
        assert_eq!(read_version(&android, ProjectKind::NativeAndroid).as_deref(), Some("3.1.5+30105000"));
    }

    #[test]
    fn writes_expo_ids_keeping_key_order() {
        let root = project(&[("package.json", "{}"), ("app.json", r#"{"expo":{"name":"Shop","slug":"shop","ios":{"supportsTablet":true}}}"#)]);
        assert!(write_expo_ids(&root, Some("not an id"), None).is_err());
        write_expo_ids(&root, Some("com.acme.shop"), Some("com.acme.shop")).unwrap();
        let config = read_expo_config(&root);
        assert_eq!(config.bundle_identifier.as_deref(), Some("com.acme.shop"));
        assert_eq!(config.package.as_deref(), Some("com.acme.shop"));
        let written = std::fs::read_to_string(root.join("app.json")).unwrap();
        assert!(written.find("\"name\"").unwrap() < written.find("\"slug\"").unwrap());
    }
}
