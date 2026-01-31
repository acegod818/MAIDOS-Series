use thiserror::Error;

#[derive(Error, Debug)]
pub enum P2pError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Peer connection failed: {0}")]
    PeerConnection(String),

    #[error("Message parsing error: {0}")]
    MessageParse(String),

    #[error("Discovery error: {0}")]
    Discovery(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Routing error: {0}")]
    Routing(String),
}

pub type Result<T> = std::result::Result<T, P2pError>;

impl From<serde_json::Error> for P2pError {
    fn from(err: serde_json::Error) -> Self {
        P2pError::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = P2pError::Network("down".to_string());
        assert!(err.to_string().contains("Network error"));
    }

    #[test]
    fn test_from_serde_error() {
        let err: P2pError = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err().into();
        assert!(matches!(err, P2pError::Serialization(_)));
    }
}
