use regex::Regex;
use std::collections::BTreeMap;
use std::path::Path;

fn colors_dart_path(workspace: &str, project_id: &str) -> std::path::PathBuf {
    Path::new(workspace).join("lib/conf/sermobplus-projects").join(project_id).join("colors.dart")
}

fn flutter_color_to_hex(flutter_value: &str) -> String {
    let re = Regex::new(r"Color\(0xFF([0-9A-Fa-f]{6})\)").unwrap();
    re.captures(flutter_value)
        .map(|c| format!("#{}", c[1].to_uppercase()))
        .unwrap_or_else(|| "#FFFFFF".to_string())
}

fn hex_to_flutter_color(hex: &str) -> String {
    let clean = hex.trim_start_matches('#').to_uppercase();
    format!("Color(0xFF{clean})")
}

fn parse_colors_dart(content: &str) -> BTreeMap<String, String> {
    let re = Regex::new(r"const\s+(\w+)\s*=\s*(Color\(0xFF[0-9A-Fa-f]{6}\))\s*;").unwrap();
    re.captures_iter(content).map(|c| (c[1].to_string(), flutter_color_to_hex(&c[2]))).collect()
}

pub fn read_colors(workspace: &str, project_id: &str) -> BTreeMap<String, String> {
    let path = colors_dart_path(workspace, project_id);
    match std::fs::read_to_string(&path) {
        Ok(content) => parse_colors_dart(&content),
        Err(_) => BTreeMap::from([
            ("fallbackPrimary".to_string(), "#FFFFFF".to_string()),
            ("fallbackAccent".to_string(), "#FFFBFA".to_string()),
            ("fallbackDark".to_string(), "#37474F".to_string()),
        ]),
    }
}

pub fn write_colors(workspace: &str, project_id: &str, colors: &BTreeMap<String, String>) -> std::io::Result<()> {
    let path = colors_dart_path(workspace, project_id);
    let mut content = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        "import 'package:flutter/material.dart';\n\nconst fallbackPrimary = Color(0xFFFFFFFF);\nconst fallbackAccent = Color(0xFFFFFBFA);\nconst fallbackDark = Color(0xFF37474F);\n"
            .to_string()
    });

    for (key, hex) in colors {
        let flutter_color = hex_to_flutter_color(hex);
        let re = Regex::new(&format!(r"(const\s+{key}\s*=\s*)Color\(0xFF[0-9A-Fa-f]{{6}}\)(\s*;)")).unwrap();
        if re.is_match(&content) {
            content = re.replace_all(&content, format!("${{1}}{flutter_color}${{2}}").as_str()).to_string();
        } else {
            content.push_str(&format!("const {key} = {flutter_color};\n"));
        }
    }

    std::fs::write(path, content)
}
