use crate::jwk::JsonWebKey;
use crate::jwk::JsonWebKeySet;
#[cfg(feature = "async")]
use async_trait::async_trait;
use headers::{Header, HeaderMap};
use reqwest::header::CACHE_CONTROL;
use std::time::Instant;

const GOOGLE_CERT_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";

#[cfg(feature = "blocking")]
pub trait KeyProvider {
    fn get_key(&mut self, key_id: &str) -> Result<Option<JsonWebKey>, ()>;
}

#[cfg(feature = "async")]
#[async_trait]
pub trait AsyncKeyProvider {
    async fn get_key_async(&mut self, key_id: &str) -> Result<Option<JsonWebKey>, ()>;
}

#[derive(Clone)]
pub struct GoogleKeyProvider {
    cached: Option<JsonWebKeySet>,
    expiration_time: Instant,
}

impl Default for GoogleKeyProvider {
    fn default() -> Self {
        Self {
            cached: None,
            expiration_time: Instant::now(),
        }
    }
}

impl GoogleKeyProvider {
    fn process_response(&mut self, headers: &HeaderMap, text: &str) -> Result<&JsonWebKeySet, ()> {
        let mut expiration_time = None;
        let x = headers.get_all(CACHE_CONTROL);
        if let Ok(cache_header) = headers::CacheControl::decode(&mut x.iter()) {
            if let Some(max_age) = cache_header.max_age() {
                expiration_time = Some(Instant::now() + max_age);
            }
        }
        let key_set = serde_json::from_str(&text).map_err(|_| ())?;
        if let Some(expiration_time) = expiration_time {
            self.cached = Some(key_set);
            self.expiration_time = expiration_time;
        }
        Ok(self.cached.as_ref().unwrap())
    }
    #[cfg(feature = "blocking")]
    pub fn download_keys(&mut self) -> Result<&JsonWebKeySet, ()> {
        let result = reqwest::blocking::get(GOOGLE_CERT_URL).map_err(|_| ())?;
        self.process_response(&result.headers().clone(), &result.text().map_err(|_| ())?)
    }
    #[cfg(feature = "async")]
    async fn download_keys_async(&mut self) -> Result<&JsonWebKeySet, ()> {
        let result = reqwest::get(GOOGLE_CERT_URL).await.map_err(|_| ())?;
        self.process_response(
            &result.headers().clone(),
            &result.text().await.map_err(|_| ())?,
        )
    }
}

#[cfg(feature = "blocking")]
impl KeyProvider for GoogleKeyProvider {
    fn get_key(&mut self, key_id: &str) -> Result<Option<JsonWebKey>, ()> {
        if let Some(ref cached_keys) = self.cached {
            if self.expiration_time > Instant::now() {
                return Ok(cached_keys.get_key(key_id));
            }
        }
        Ok(self.download_keys()?.get_key(key_id))
    }
}

#[cfg(feature = "async")]
#[async_trait]
impl AsyncKeyProvider for GoogleKeyProvider {
    async fn get_key_async(&mut self, key_id: &str) -> Result<Option<JsonWebKey>, ()> {
        if let Some(ref cached_keys) = self.cached {
            if self.expiration_time > Instant::now() {
                return Ok(cached_keys.get_key(key_id));
            }
        }
        Ok(self.download_keys_async().await?.get_key(key_id))
    }
}

#[cfg(feature = "blocking")]
#[test]
pub fn test_google_provider() {
    let mut provider = GoogleKeyProvider::default();
    assert!(provider.get_key("test").is_ok());
    assert!(provider.get_key("test").is_ok());
}

#[cfg(all(test, feature = "async"))]
mod async_test {
    use super::{AsyncKeyProvider, GoogleKeyProvider};
    use tokio;
    #[tokio::test]
    async fn test_google_provider_async() {
        let mut provider = GoogleKeyProvider::default();
        assert!(provider.get_key_async("test").await.is_ok());
        assert!(provider.get_key_async("test").await.is_ok());
    }
}
