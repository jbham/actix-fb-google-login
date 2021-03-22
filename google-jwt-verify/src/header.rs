use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct Header {
    #[serde(rename = "kid")]
    pub key_id: String,
}
