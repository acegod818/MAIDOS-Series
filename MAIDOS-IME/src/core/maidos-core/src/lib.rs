//! MAIDOS-IME core module
//!
//! This module provides the core functionality of the IME engine, including:
//! - Input scheme processing
//! - AI character selection
//! - Context understanding
//! - Auto-correction
//! - Smart suggestions

pub mod ime_engine;
pub mod schemes;
pub mod ai;
pub mod converter;
pub mod dictionary;
pub mod pinyin_parser;
pub mod english;
pub mod japanese;
pub mod user_learning;
pub mod ffi;

/// MAIDOS-IME core error type
#[derive(thiserror::Error, Debug)]
pub enum MaidosError {
    #[error("Config error: {0}")]
    ConfigError(String),
    #[error("AI model error: {0}")]
    AiModelError(String),
    #[error("Input scheme error: {0}")]
    SchemeError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl From<maidos_config::ConfigError> for MaidosError {
    fn from(err: maidos_config::ConfigError) -> Self {
        MaidosError::ConfigError(format!("{:?}", err))
    }
}

impl From<maidos_llm::LlmError> for MaidosError {
    fn from(err: maidos_llm::LlmError) -> Self {
        MaidosError::AiModelError(format!("{:?}", err))
    }
}

/// MAIDOS-IME core result type
pub type Result<T> = std::result::Result<T, MaidosError>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
