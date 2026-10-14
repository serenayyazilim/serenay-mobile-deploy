//! Runs a project's unit tests with its own runner and reports each test's outcome:
//! `flutter test`, Jest (React Native / Expo), Gradle (Android) and `xcodebuild test` (iOS).

use super::flutter::run_logged_command_capture;
use crate::setup::{self, ProjectKind};
use regex::Regex;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TestCase {
    pub name: String,
    pub status: TestStatus,
    pub message: Option<String>,
}

/// Starts the test run and streams `test-run-event-{jobId}` events: `log` lines, then a
/// `done` event carrying the parsed `results`.
#[tauri::command]
pub async fn tests_run_start(app: AppHandle, workspace_path: String) -> Result<String, String> {
    let root = PathBuf::from(&workspace_path);
    let kind = setup::detect_kind(&root).ok_or("Unrecognized project type")?;
    let report = std::env::temp_dir().join(format!("serenay-tests-{}.json", uuid::Uuid::new_v4()));
    let (program, args) = test_command(&root, kind, &report)?;

    let job_id = uuid::Uuid::new_v4().to_string();
    let event_name = format!("test-run-event-{job_id}");
    tokio::spawn(async move {
        let _ = app.emit(&event_name, json!({ "type": "log", "message": format!("$ {program} {}", args.join(" ")) }));
        let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
        let (ok, stdout) = match run_logged_command_capture(&program, &args_ref, &workspace_path, &app, &event_name, "", None).await {
            Ok(r) => r,
            Err(e) => {
                let _ = app.emit(&event_name, json!({ "type": "done", "success": false, "message": format!("Could not start {program}: {e}"), "results": [] }));
                return;
            }
        };

        let report_text = std::fs::read_to_string(&report).unwrap_or_default();
        let _ = std::fs::remove_file(&report);
        let results = match kind {
            ProjectKind::Flutter => parse_flutter_json(&report_text),
            ProjectKind::ReactNative | ProjectKind::Expo => parse_jest_json(&report_text),
            ProjectKind::NativeAndroid => parse_junit_dir(&root),
            ProjectKind::NativeIos => parse_xcodebuild(&stdout),
        };
        let failed = results.iter().filter(|t| t.status == TestStatus::Failed).count();
        let message = if results.is_empty() && !ok {
            "Test run failed — see logs".to_string()
        } else {
            format!("{} tests, {failed} failed", results.len())
        };
        let _ = app.emit(&event_name, json!({ "type": "done", "success": ok && failed == 0, "message": message, "results": results }));
    });
    Ok(job_id)
}

fn test_command(root: &Path, kind: ProjectKind, report: &Path) -> Result<(String, Vec<String>), String> {
    let report = report.to_string_lossy();
    Ok(match kind {
        ProjectKind::Flutter => ("flutter".into(), vec!["test".into(), "--file-reporter".into(), format!("json:{report}")]),
        ProjectKind::ReactNative | ProjectKind::Expo => {
            let uses_jest = setup::package_json(root).is_some_and(|p| {
                ["jest", "jest-expo"].iter().any(|d| setup::has_dependency(&p, d))
                    || p.pointer("/scripts/test").and_then(Value::as_str).is_some_and(|s| s.contains("jest"))
            });
            if !uses_jest {
                return Err("No Jest setup found in package.json".into());
            }
            ("npx".into(), vec!["jest".into(), "--ci".into(), "--json".into(), format!("--outputFile={report}")])
        }
        // `--continue` so one module's failures don't hide the others' results.
        ProjectKind::NativeAndroid => ("./gradlew".into(), vec!["test".into(), "--continue".into()]),
        ProjectKind::NativeIos => {
            let mut args = vec!["test".to_string()];
            let project = setup::find_xcode_file(root, "xcodeproj").ok_or("No .xcodeproj found")?;
            match setup::find_xcode_file(root, "xcworkspace") {
                Some(ws) => args.extend(["-workspace".into(), ws.to_string_lossy().into()]),
                None => args.extend(["-project".into(), project.to_string_lossy().into()]),
            }
            // ponytail: scheme = project name (Xcode's default); add a scheme picker if projects differ.
            let scheme = project.file_stem().unwrap_or_default().to_string_lossy().to_string();
            let udid = ios_simulator().ok_or("No iOS simulator available")?;
            args.extend(["-scheme".into(), scheme, "-destination".into(), format!("id={udid}")]);
            ("xcodebuild".into(), args)
        }
    })
}

/// A booted iPhone simulator if there is one, else the first available iPhone.
fn ios_simulator() -> Option<String> {
    let out = std::process::Command::new("xcrun").args(["simctl", "list", "devices", "available", "--json"]).output().ok()?;
    let v: Value = serde_json::from_slice(&out.stdout).ok()?;
    let iphones: Vec<&Value> = v["devices"]
        .as_object()?
        .iter()
        .filter(|(runtime, _)| runtime.contains("iOS"))
        .flat_map(|(_, devices)| devices.as_array().into_iter().flatten())
        .filter(|d| d["name"].as_str().is_some_and(|n| n.starts_with("iPhone")))
        .collect();
    let pick = iphones.iter().find(|d| d["state"] == "Booted").or_else(|| iphones.last())?;
    pick["udid"].as_str().map(String::from)
}

/// `flutter test --file-reporter json:…` — one JSON event per line.
fn parse_flutter_json(text: &str) -> Vec<TestCase> {
    let mut names: HashMap<i64, String> = HashMap::new();
    let mut errors: HashMap<i64, String> = HashMap::new();
    let mut out = Vec::new();
    for v in text.lines().filter_map(|l| serde_json::from_str::<Value>(l).ok()) {
        match v["type"].as_str() {
            Some("testStart") => {
                if let (Some(id), Some(name)) = (v["test"]["id"].as_i64(), v["test"]["name"].as_str()) {
                    names.insert(id, name.to_string());
                }
            }
            Some("error") => {
                if let Some(id) = v["testID"].as_i64() {
                    let e = errors.entry(id).or_default();
                    e.push_str(v["error"].as_str().unwrap_or_default());
                    e.push('\n');
                }
            }
            Some("testDone") => {
                let Some(id) = v["testID"].as_i64() else { continue };
                let passed = v["result"] == "success";
                // Hidden tests are the runner's own "loading <file>" steps; only their failures matter.
                if v["hidden"] == true && passed {
                    continue;
                }
                let status = if v["skipped"] == true { TestStatus::Skipped } else if passed { TestStatus::Passed } else { TestStatus::Failed };
                out.push(TestCase { name: names.remove(&id).unwrap_or_default(), status, message: errors.remove(&id).map(|e| e.trim().to_string()) });
            }
            _ => {}
        }
    }
    out
}

/// Jest's `--json` report. A suite that failed to run has no assertions, only a message.
fn parse_jest_json(text: &str) -> Vec<TestCase> {
    let Ok(v) = serde_json::from_str::<Value>(text) else { return Vec::new() };
    let mut out = Vec::new();
    for suite in v["testResults"].as_array().into_iter().flatten() {
        let assertions = suite["assertionResults"].as_array().cloned().unwrap_or_default();
        if assertions.is_empty() && suite["status"] == "failed" {
            out.push(TestCase { name: suite["name"].as_str().unwrap_or_default().to_string(), status: TestStatus::Failed, message: suite["message"].as_str().map(String::from) });
        }
        for a in assertions {
            let status = match a["status"].as_str() {
                Some("passed") => TestStatus::Passed,
                Some("failed") => TestStatus::Failed,
                _ => TestStatus::Skipped,
            };
            let message = a["failureMessages"].as_array().filter(|m| !m.is_empty()).map(|m| m.iter().filter_map(Value::as_str).collect::<Vec<_>>().join("\n"));
            out.push(TestCase { name: a["fullName"].as_str().unwrap_or_default().to_string(), status, message });
        }
    }
    out
}

/// Gradle's JUnit XML reports under every module's `build/test-results/`. The same test
/// runs once per build variant, so results are merged by name with a failure winning.
fn parse_junit_dir(root: &Path) -> Vec<TestCase> {
    let mut by_name: BTreeMap<String, TestCase> = BTreeMap::new();
    for file in junit_files(root) {
        for case in parse_junit(&std::fs::read_to_string(file).unwrap_or_default()) {
            if by_name.get(&case.name).is_none_or(|_| case.status == TestStatus::Failed) {
                by_name.insert(case.name.clone(), case);
            }
        }
    }
    by_name.into_values().collect()
}

fn junit_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for path in std::fs::read_dir(&dir).into_iter().flatten().flatten().map(|e| e.path()) {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if path.is_dir() && !name.starts_with('.') && name != "node_modules" {
                stack.push(path);
            } else if name.ends_with(".xml") && path.components().any(|c| c.as_os_str() == "test-results") {
                files.push(path);
            }
        }
    }
    files
}

// ponytail: regex over Gradle's flat JUnit XML instead of an XML parser; swap in quick-xml if reports get nested.
fn parse_junit(xml: &str) -> Vec<TestCase> {
    let case_re = Regex::new(r#"(?s)<testcase\b([^>]*?)(?:/>|>(.*?)</testcase>)"#).unwrap();
    let attr = |attrs: &str, key: &str| {
        Regex::new(&format!(r#"\b{key}="([^"]*)""#)).unwrap().captures(attrs).map(|c| unescape_xml(&c[1]))
    };
    case_re
        .captures_iter(xml)
        .map(|c| {
            let body = c.get(2).map_or("", |m| m.as_str());
            let (status, message) = if body.contains("<failure") || body.contains("<error") {
                let msg = Regex::new(r#"(?s)<(?:failure|error)\b[^>]*>(.*?)</(?:failure|error)>"#).unwrap().captures(body).map(|m| unescape_xml(m[1].trim()));
                (TestStatus::Failed, msg)
            } else if body.contains("<skipped") {
                (TestStatus::Skipped, None)
            } else {
                (TestStatus::Passed, None)
            };
            let class = attr(&c[1], "classname").unwrap_or_default();
            let name = attr(&c[1], "name").unwrap_or_default();
            TestCase { name: format!("{class} › {name}"), status, message }
        })
        .collect()
}

fn unescape_xml(s: &str) -> String {
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&#10;", "\n").replace("&amp;", "&")
}

/// XCTest (`Test Case '-[T.C testX]' passed`) and Swift Testing (`✔ Test x() passed`) lines.
/// Failure messages are the `error:` lines that name the test.
fn parse_xcodebuild(lines: &[String]) -> Vec<TestCase> {
    let result_re = Regex::new(r"^(?:Test [Cc]ase '(.+?)'|[✔✘➜] Test (.+?)) (passed|failed|skipped)").unwrap();
    let error_re = Regex::new(r": error: (.+?) : (.+)$").unwrap();
    let mut errors: HashMap<String, String> = HashMap::new();
    let mut out = Vec::new();
    for line in lines {
        if let Some(c) = error_re.captures(line) {
            errors.entry(c[1].trim().to_string()).or_default().push_str(&format!("{}\n", &c[2]));
        } else if let Some(c) = result_re.captures(line) {
            let name = c.get(1).or(c.get(2)).unwrap().as_str().to_string();
            let status = match &c[3] {
                "passed" => TestStatus::Passed,
                "failed" => TestStatus::Failed,
                _ => TestStatus::Skipped,
            };
            out.push(TestCase { message: errors.remove(&name).map(|e| e.trim().to_string()), name, status });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_each_runner_report() {
        let flutter = r#"{"type":"testStart","test":{"id":1,"name":"loading /a_test.dart"}}
{"type":"testDone","testID":1,"result":"success","hidden":true,"skipped":false}
{"type":"testStart","test":{"id":2,"name":"adds"}}
{"type":"testDone","testID":2,"result":"success","hidden":false,"skipped":false}
{"type":"testStart","test":{"id":3,"name":"subtracts"}}
{"type":"error","testID":3,"error":"Expected: 1","isFailure":true}
{"type":"testDone","testID":3,"result":"failure","hidden":false,"skipped":false}"#;
        assert_eq!(
            parse_flutter_json(flutter),
            [
                TestCase { name: "adds".into(), status: TestStatus::Passed, message: None },
                TestCase { name: "subtracts".into(), status: TestStatus::Failed, message: Some("Expected: 1".into()) },
            ]
        );

        let jest = r#"{"testResults":[
            {"name":"/a.test.js","status":"failed","assertionResults":[
                {"fullName":"math adds","status":"passed","failureMessages":[]},
                {"fullName":"math subtracts","status":"failed","failureMessages":["boom"]},
                {"fullName":"math later","status":"pending","failureMessages":[]}]},
            {"name":"/b.test.js","status":"failed","message":"SyntaxError","assertionResults":[]}]}"#;
        let statuses: Vec<_> = parse_jest_json(jest).into_iter().map(|t| (t.name, t.status)).collect();
        assert_eq!(statuses, [
            ("math adds".into(), TestStatus::Passed),
            ("math subtracts".into(), TestStatus::Failed),
            ("math later".into(), TestStatus::Skipped),
            ("/b.test.js".into(), TestStatus::Failed),
        ]);

        let junit = r#"<testsuite><testcase name="adds" classname="a.MathTest" time="0"/>
<testcase name="subtracts" classname="a.MathTest" time="0"><failure message="x" type="AssertionError">expected:&lt;1&gt;</failure></testcase>
<testcase name="later" classname="a.MathTest"><skipped/></testcase></testsuite>"#;
        let cases = parse_junit(junit);
        assert_eq!(cases[0], TestCase { name: "a.MathTest › adds".into(), status: TestStatus::Passed, message: None });
        assert_eq!(cases[1].message.as_deref(), Some("expected:<1>"));
        assert_eq!(cases[2].status, TestStatus::Skipped);

        let xcode: Vec<String> = [
            "Test Case '-[AppTests.MathTests testAdds]' passed (0.001 seconds).",
            "/x/MathTests.swift:9: error: -[AppTests.MathTests testSubtracts] : XCTAssertEqual failed",
            "Test Case '-[AppTests.MathTests testSubtracts]' failed (0.002 seconds).",
            "✔ Test multiplies() passed after 0.001 seconds.",
        ].map(String::from).to_vec();
        let cases = parse_xcodebuild(&xcode);
        assert_eq!(cases.len(), 3);
        assert_eq!(cases[1].message.as_deref(), Some("XCTAssertEqual failed"));
        assert_eq!(cases[2], TestCase { name: "multiplies()".into(), status: TestStatus::Passed, message: None });
    }
}
