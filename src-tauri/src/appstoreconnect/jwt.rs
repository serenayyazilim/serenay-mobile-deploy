use super::config::AscConfig;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;

// App Store Connect API tokens must expire within 20 minutes.
const TOKEN_TTL_SECONDS: i64 = 19 * 60;

#[derive(Serialize)]
struct Claims {
    iss: String,
    aud: String,
    iat: i64,
    exp: i64,
}

pub fn generate_asc_token(config: &AscConfig) -> Result<String, String> {
    let now = chrono::Utc::now().timestamp();
    let claims = Claims {
        iss: config.issuer_id.clone(),
        aud: "appstoreconnect-v1".to_string(),
        iat: now,
        exp: now + TOKEN_TTL_SECONDS,
    };

    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(config.key_id.clone());

    let key = EncodingKey::from_ec_pem(config.private_key.as_bytes()).map_err(|e| e.to_string())?;
    encode(&header, &claims, &key).map_err(|e| e.to_string())
}
