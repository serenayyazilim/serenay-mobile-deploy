use regex::Regex;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const KNOWN_ENUM_PREFIXES: [&str; 2] = ["LoginType.", "GridCardTur."];

pub fn serconf_path(workspace: &str, project_id: &str) -> PathBuf {
    Path::new(workspace).join("lib/conf/sermobplus-projects").join(project_id).join("serconf.dart")
}

fn parse_value(raw: &str) -> Value {
    let trimmed = raw.trim();

    if trimmed == "true" {
        return Value::Bool(true);
    }
    if trimmed == "false" {
        return Value::Bool(false);
    }

    if Regex::new(r"^-?\d+$").unwrap().is_match(trimmed) {
        if let Ok(n) = trimmed.parse::<i64>() {
            return Value::Number(n.into());
        }
    }
    if Regex::new(r"^-?\d+\.\d+$").unwrap().is_match(trimmed) {
        if let Ok(n) = trimmed.parse::<f64>() {
            if let Some(num) = serde_json::Number::from_f64(n) {
                return Value::Number(num);
            }
        }
    }

    // Division expression (e.g., 1200 / 1600) — keep as string for display
    if Regex::new(r"^\d+\s*/\s*\d+$").unwrap().is_match(trimmed) {
        return Value::String(trimmed.to_string());
    }

    let string_re = Regex::new(r#"^['"](.*)['"]$"#).unwrap();
    if let Some(c) = string_re.captures(trimmed) {
        return Value::String(c[1].to_string());
    }

    // Enum or other identifier (keep as-is)
    Value::String(trimmed.to_string())
}

fn to_dart_value(value: &Value) -> String {
    match value {
        Value::Bool(b) => (if *b { "true" } else { "false" }).to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            if KNOWN_ENUM_PREFIXES.iter().any(|p| s.starts_with(p)) {
                return s.clone();
            }
            if Regex::new(r"^\d+\s*/\s*\d+$").unwrap().is_match(s) {
                return s.clone();
            }
            format!("'{}'", s.replace('\'', "\\'"))
        }
        _ => value.to_string(),
    }
}

fn parse_dart_serconf(content: &str) -> BTreeMap<String, Value> {
    let re = Regex::new(r"const\s+(\w+)\s*=\s*([^;]+);").unwrap();
    re.captures_iter(content).map(|c| (c[1].to_string(), parse_value(&c[2]))).collect()
}

fn update_dart_const(content: &str, key: &str, value: &Value) -> String {
    let re = Regex::new(&format!(r"(const\s+{key}\s*=\s*)([^;]+)(;)")).unwrap();
    let dart_value = to_dart_value(value);

    if !re.is_match(content) {
        let mut new_content = content.trim_end().to_string();
        new_content.push_str(&format!("\n\nconst {key} = {dart_value};\n"));
        return new_content;
    }

    re.replace_all(content, format!("${{1}}{dart_value}${{3}}").as_str()).to_string()
}

pub fn read_serconf(workspace: &str, project_id: &str) -> Option<BTreeMap<String, Value>> {
    let content = std::fs::read_to_string(serconf_path(workspace, project_id)).ok()?;
    let mut config = parse_dart_serconf(&content);

    for key in ["API_URL", "ILETISIM_URL"] {
        if let Some(Value::String(val)) = config.get_mut(key) {
            let mut v = val.clone();
            v = Regex::new(r"^https?://(www\.)?").unwrap().replace(&v, "").to_string();
            v = Regex::new(r"/sermobileboss/?$").unwrap().replace(&v, "").to_string();
            *val = v;
        }
    }

    Some(config)
}

pub fn write_serconf(workspace: &str, project_id: &str, updates: &BTreeMap<String, Value>) -> Result<(), String> {
    let path = serconf_path(workspace, project_id);
    let mut content = std::fs::read_to_string(&path).map_err(|_| "serconf.dart not found".to_string())?;

    for (key, value) in updates {
        let final_value = if key == "API_URL" || key == "ILETISIM_URL" {
            match value {
                Value::String(s) if !s.is_empty() && !s.starts_with("http") => {
                    Value::String(format!("https://{s}/sermobileboss/"))
                }
                other => other.clone(),
            }
        } else {
            value.clone()
        };

        content = update_dart_const(&content, key, &final_value);
    }

    std::fs::write(&path, content).map_err(|e| e.to_string())
}
