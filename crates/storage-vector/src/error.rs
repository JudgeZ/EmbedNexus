use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("I/O error: {0}")]
    Io(String),
    #[error("quota error: {0}")]
    Quota(String),
    #[error("ledger error: {0}")]
    Ledger(String),
    #[error("integrity error: {0}")]
    Integrity(String),
    #[cfg(feature = "encryption")]
    #[error("encryption error: {0}")]
    Encryption(String),
    #[cfg(feature = "encryption")]
    #[error("key manager error: {0}")]
    Key(String),
    #[error("unsupported operation: {0}")]
    Unsupported(String),
}

