use serde_derive::Deserialize;

pub struct Token<P> {
    required_claims: RequiredClaims,
    payload: P,
}

impl<P> Token<P> {
    pub fn new(required_claims: RequiredClaims, payload: P) -> Token<P> {
        Token {
            required_claims,
            payload,
        }
    }
    pub fn get_claims(&self) -> RequiredClaims {
        self.required_claims.clone()
    }
    pub fn get_payload(&self) -> &P {
        &self.payload
    }
}

#[derive(Deserialize, Clone)]
pub struct RequiredClaims {
    #[serde(rename = "iss")]
    issuer: String,

    #[serde(rename = "sub")]
    subject: String,

    #[serde(rename = "aud")]
    audience: String,

    #[serde(rename = "azp")]
    android_audience: String,

    #[serde(rename = "iat")]
    issued_at: u64,

    #[serde(rename = "exp")]
    expires_at: u64,
}

impl RequiredClaims {
    pub fn get_issuer(&self) -> String {
        self.issuer.clone()
    }
    pub fn get_subject(&self) -> String {
        self.subject.clone()
    }
    pub fn get_audience(&self) -> String {
        self.audience.clone()
    }
    pub fn get_android_audience(&self) -> String {
        self.android_audience.clone()
    }
    pub fn get_issued_at(&self) -> u64 {
        self.issued_at
    }
    pub fn get_expires_at(&self) -> u64 {
        self.expires_at
    }
}

#[derive(Deserialize, Clone)]
pub struct IdPayload {
    email: String,
    email_verified: bool,
    name: String,
    picture: String,
    given_name: String,
    family_name: String,
    locale: String,
    hd: Option<String>,
}

impl IdPayload {
    pub fn get_email(&self) -> String {
        self.email.clone()
    }
    pub fn is_email_verified(&self) -> bool {
        self.email_verified
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_picture_url(&self) -> String {
        self.picture.clone()
    }
    pub fn get_given_name(&self) -> String {
        self.given_name.clone()
    }
    pub fn get_family_name(&self) -> String {
        self.family_name.clone()
    }
    pub fn get_locale(&self) -> String {
        self.locale.clone()
    }
    pub fn get_domain(&self) -> Option<String> {
        self.hd.clone()
    }
}
