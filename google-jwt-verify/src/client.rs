use crate::error::Error;
#[cfg(feature = "async")]
use crate::key_provider::AsyncKeyProvider;
use crate::key_provider::GoogleKeyProvider;
#[cfg(feature = "blocking")]
use crate::key_provider::KeyProvider;
use crate::token::IdPayload;
use crate::token::Token;
use crate::unverified_token::UnverifiedToken;
use serde::Deserialize;

use std::sync::{Arc, Mutex};

pub type Client = GenericClient<GoogleKeyProvider>;

pub struct GenericClientBuilder<KP> {
    client_id: String,
    key_provider: Arc<Mutex<KP>>,
    check_expiration: bool,
}

impl<KP: Default> GenericClientBuilder<KP> {
    pub fn new(client_id: &str) -> GenericClientBuilder<KP> {
        GenericClientBuilder::<KP> {
            client_id: client_id.to_owned(),
            key_provider: Arc::new(Mutex::new(KP::default())),
            check_expiration: true,
        }
    }
}

impl<KP> GenericClientBuilder<KP> {
    pub fn custom_key_provider<T>(self, provider: T) -> GenericClientBuilder<T> {
        GenericClientBuilder {
            client_id: self.client_id,
            key_provider: Arc::new(Mutex::new(provider)),
            check_expiration: self.check_expiration,
        }
    }
    pub fn unsafe_ignore_expiration(mut self) -> Self {
        self.check_expiration = false;
        self
    }
    pub fn build(self) -> GenericClient<KP> {
        GenericClient {
            client_id: self.client_id,
            key_provider: self.key_provider,
            check_expiration: self.check_expiration,
        }
    }
}

pub struct GenericClient<T> {
    client_id: String,
    key_provider: Arc<Mutex<T>>,
    check_expiration: bool,
}

impl<KP: Default> GenericClient<KP> {
    pub fn builder(client_id: &str) -> GenericClientBuilder<KP> {
        GenericClientBuilder::<KP>::new(client_id)
    }
    pub fn new(client_id: &str) -> GenericClient<KP> {
        GenericClientBuilder::new(client_id).build()
    }
}

#[cfg(feature = "blocking")]
impl<KP: KeyProvider> GenericClient<KP> {
    pub fn verify_token_with_payload<P>(&self, token_string: &str) -> Result<Token<P>, Error>
    where
        for<'a> P: Deserialize<'a>,
    {
        let unverified_token =
            UnverifiedToken::<P>::validate(token_string, self.check_expiration, &self.client_id)?;
        unverified_token.verify(&self.key_provider)
    }

    pub fn verify_token(&self, token_string: &str) -> Result<Token<()>, Error> {
        self.verify_token_with_payload::<()>(token_string)
    }

    pub fn verify_id_token(&self, token_string: &str) -> Result<Token<IdPayload>, Error> {
        self.verify_token_with_payload(token_string)
    }
}

#[cfg(feature = "async")]
impl<KP: AsyncKeyProvider> GenericClient<KP> {
    pub async fn verify_token_with_payload_async<P>(
        &self,
        token_string: &str,
    ) -> Result<Token<P>, Error>
    where
        for<'a> P: Deserialize<'a>,
    {
        let unverified_token =
            UnverifiedToken::<P>::validate(token_string, self.check_expiration, &self.client_id)?;
        unverified_token.verify_async(&self.key_provider).await
    }

    pub async fn verify_token_async(&self, token_string: &str) -> Result<Token<()>, Error> {
        self.verify_token_with_payload_async::<()>(token_string)
            .await
    }

    pub async fn verify_id_token_async(
        &self,
        token_string: &str,
    ) -> Result<Token<IdPayload>, Error> {
        self.verify_token_with_payload_async(token_string).await
    }
}
