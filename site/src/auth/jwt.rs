use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rsa::{pkcs8::DecodePrivateKey, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub iat: i64,
    pub exp: i64,
}

pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
    pub jwks_json: String,
}

impl JwtKeys {
    pub fn from_pem(private_pem: &str, public_pem: &str) -> Result<Self> {
        let private_key = RsaPrivateKey::from_pkcs8_pem(private_pem)?;
        let public_key = RsaPublicKey::from(private_key.clone());

        let encoding = EncodingKey::from_rsa_pem(private_pem.as_bytes())?;
        let decoding = DecodingKey::from_rsa_pem(public_pem.as_bytes())?;

        let jwks_json = build_jwks(&public_key)?;

        Ok(Self { encoding, decoding, jwks_json })
    }

    pub fn sign(&self, user_id: Uuid, username: &str, ttl_secs: i64) -> Result<String> {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            iat: now,
            exp: now + ttl_secs,
        };
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("beyvox-1".to_string());
        Ok(encode(&header, &claims, &self.encoding)?)
    }

    pub fn verify(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&["beyvox"]);
        let data = decode::<Claims>(token, &self.decoding, &validation)?;
        Ok(data.claims)
    }
}

fn build_jwks(public_key: &RsaPublicKey) -> Result<String> {
    use rsa::traits::PublicKeyParts;

    let n = URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be());
    let e = URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be());

    let key_json = serde_json::json!({
        "kty": "RSA",
        "use": "sig",
        "alg": "RS256",
        "kid": "beyvox-1",
        "n": n,
        "e": e,
    });

    let mut hasher = Sha256::new();
    hasher.update(n.as_bytes());
    let _ = hasher.finalize();

    Ok(serde_json::json!({ "keys": [key_json] }).to_string())
}
