use pgp::SecretKeyParamsBuilderError;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("PGP Error: {0}")]
    PgpError(#[from] pgp::errors::Error),

    #[error("Key Pair Error: {0}")]
    KeyPairError(#[from] SecretKeyParamsBuilderError),

    #[error("Unknown Error: {0}")]
    UnknownError(String),
}

impl Serialize for LibError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}