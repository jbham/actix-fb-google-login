use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub enum Algorithm {
    RS256,
    RS384,
    RS512,
    HS256,
    HS384,
    HS512,
    ES256,
    ES384,
    ES512,
}
