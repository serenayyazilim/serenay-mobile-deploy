use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

const LOCALE_TO_LANG: &[(&str, &str)] = &[
    ("tr", "tr"), ("tr-TR", "tr"),
    ("en-US", "en"),
    ("ru", "ru"), ("ru-RU", "ru"),
    ("fr-FR", "fr"),
    ("it", "it"), ("it-IT", "it"),
    ("ar-SA", "ar"), ("ar", "ar"),
    ("es-ES", "es"),
    ("kk", "kk"),
];

fn locale_to_lang(locale: &str) -> String {
    LOCALE_TO_LANG.iter().find(|(l, _)| *l == locale).map(|(_, lang)| lang.to_string()).unwrap_or_else(|| locale.to_string())
}

async fn translate_text(text: &str, target_lang: &str) -> String {
    let url = format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={}&dt=t&q={}",
        target_lang,
        urlencoding(text)
    );

    let client = match reqwest::Client::builder().timeout(Duration::from_secs(8)).build() {
        Ok(c) => c,
        Err(_) => return text.to_string(),
    };

    let Ok(res) = client.get(&url).send().await else { return text.to_string() };
    if !res.status().is_success() {
        return text.to_string();
    }
    let Ok(data) = res.json::<Value>().await else { return text.to_string() };

    data.get(0)
        .and_then(|v| v.as_array())
        .map(|segments| {
            segments
                .iter()
                .filter_map(|s| s.get(0).and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("")
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| text.to_string())
}

pub async fn build_translations(text: &str, ios: &[String], android: &[String]) -> HashMap<String, String> {
    let all_locales: HashSet<String> = ios.iter().chain(android.iter()).cloned().collect();
    let mut lang_to_translation: HashMap<String, String> = HashMap::new();
    let mut translations = HashMap::new();

    for locale in &all_locales {
        let lang = locale_to_lang(locale);
        if !lang_to_translation.contains_key(&lang) {
            let translated = translate_text(text, &lang).await;
            lang_to_translation.insert(lang.clone(), translated);
        }
        translations.insert(locale.clone(), lang_to_translation[&lang].clone());
    }

    translations
}

fn urlencoding(s: &str) -> String {
    let mut out = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(byte as char),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
