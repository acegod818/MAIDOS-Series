use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChainError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Wallet error: {0}")]
    Wallet(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Chain not supported: {0}")]
    ChainNotSupported(String),

    #[error("Provider not configured: {0}")]
    ProviderNotConfigured(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, ChainError>;

impl From<serde_json::Error> for ChainError {
    fn from(err: serde_json::Error) -> Self {
        ChainError::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ChainError::Network("down".to_string());
        assert!(err.to_string().contains("Network error"));
    }

    #[test]
    fn test_from_serde_error() {
        let err: ChainError = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err().into();
        assert!(matches!(err, ChainError::Serialization(_)));
    }
}
