use anyhow::{anyhow, Result};
use chrono::prelude::*;
use chrono::Duration;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::{thread_rng, Rng};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

const SECRET_BYTES: usize = 64;

pub struct TokenGenerator {
    expires_after: Duration,
    secret: [u8; SECRET_BYTES],
    validation: Validation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims<T> {
    exp: i64,
    nbf: i64,
    #[serde(flatten)]
    data: T,
}

pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut arr: [u8; N] = [0; N];
    thread_rng().fill(&mut arr[..]);
    arr
}

pub fn random_string<const N: usize>() -> String {
    thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(N)
        .map(char::from)
        .collect()
}

impl TokenGenerator {
    pub fn new(expires_after: Duration) -> Result<Self> {
        Ok(Self {
            expires_after,
            secret: random_bytes::<SECRET_BYTES>(),
            validation: Validation {
                validate_exp: true,
                validate_nbf: true,
                ..Default::default()
            },
        })
    }

    pub async fn generate<T>(&self, data: T) -> Result<String>
    where
        T: Serialize,
    {
        let now = Utc::now();
        let nbf = now.timestamp();
        let exp = now
            .checked_add_signed(self.expires_after)
            .ok_or_else(|| anyhow!("Overflow while calculating login token expiry"))?
            .timestamp();
        let claims = Claims { exp, nbf, data };
        let encoding_key = EncodingKey::from_secret(&self.secret);
        Ok(encode(&Header::default(), &claims, &encoding_key)?)
    }

    pub async fn validate<T>(&self, token: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let decoding_key = DecodingKey::from_secret(&self.secret);
        let token_data = decode::<Claims<T>>(token, &decoding_key, &self.validation)?;
        Ok(token_data.claims.data)
    }
}
