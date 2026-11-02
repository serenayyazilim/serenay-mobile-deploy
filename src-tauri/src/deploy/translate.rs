use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::time::Duration;

/// `deploy_start`'s error starts with this, followed by the failed locales, until the
/// caller agrees to ship those languages with the text as written.
pub const TRANSLATION_FAILED: &str = "TRANSLATION_FAILED";

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

async fn translate_text(text: &str, target_lang: &str) -> Option<String> {
    let url = format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={}&dt=t&q={}",
        target_lang,
        urlencoding(text)
    );

    let client = reqwest::Client::builder().timeout(Duration::from_secs(8)).build().ok()?;
    let res = client.get(&url).send().await.ok()?.error_for_status().ok()?;
    parse_translation(&res.json::<Value>().await.ok()?)
}

fn parse_translation(data: &Value) -> Option<String> {
    let segments = data.get(0)?.as_array()?;
    let text: String = segments.iter().filter_map(|s| s.get(0)?.as_str()).collect();
    (!text.is_empty()).then_some(text)
}

/// Returns the translations by locale, and the locales that couldn't be translated.
pub async fn build_translations(text: &str, ios: &[String], android: &[String]) -> (HashMap<String, String>, Vec<String>) {
    let all_locales: BTreeSet<&String> = ios.iter().chain(android).collect();
    let mut by_lang: HashMap<String, Option<String>> = HashMap::new();
    let mut translations = HashMap::new();
    let mut failed = Vec::new();

    for locale in all_locales {
        let lang = locale_to_lang(locale);
        if !by_lang.contains_key(&lang) {
            let translated = translate_text(text, &lang).await;
            by_lang.insert(lang.clone(), translated);
        }
        match &by_lang[&lang] {
            Some(translated) => {
                translations.insert(locale.clone(), translated.clone());
            }
            None => failed.push(locale.clone()),
        }
    }

    (translations, failed)
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

#[cfg(test)]
mod tests {
    use super::parse_translation;
    use serde_json::json;

    #[test]
    fn joins_segments_and_rejects_unusable_responses() {
        let ok = json!([[["Hello. ", "Merhaba. "], ["World", "Dünya"]], null, "tr"]);
        assert_eq!(parse_translation(&ok).as_deref(), Some("Hello. World"));
        assert_eq!(parse_translation(&json!([[], null, "tr"])), None);
        assert_eq!(parse_translation(&json!({ "error": "nope" })), None);
    }
}
