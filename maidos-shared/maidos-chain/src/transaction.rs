use serde::{Deserialize, Serialize};

/// Simple blockchain transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas: Option<String>,
    pub gas_price: Option<String>,
    pub nonce: Option<String>,
    pub data: Option<String>,
    pub instructions: Option<serde_json::Value>, // Added for complex chain operations
}

impl Transaction {
    pub fn new(from: &str, to: &str, value: &str) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            value: value.to_string(),
            gas: None,
            gas_price: None,
            nonce: None,
            data: None,
            instructions: None,
        }
    }

    pub fn with_gas(mut self, gas: &str) -> Self {
        self.gas = Some(gas.to_string());
        self
    }

    pub fn with_gas_price(mut self, gas_price: &str) -> Self {
        self.gas_price = Some(gas_price.to_string());
        self
    }

    pub fn with_nonce(mut self, nonce: &str) -> Self {
        self.nonce = Some(nonce.to_string());
        self
    }

    pub fn with_data(mut self, data: &str) -> Self {
        self.data = Some(data.to_string());
        self
    }

    /// Validate transaction basic structure
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.from.is_empty() {
            return Err("From address cannot be empty");
        }
        if self.to.is_empty() {
            return Err("To address cannot be empty");
        }
        if self.value.is_empty() {
            return Err("Value cannot be empty");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_builders() {
        let tx = Transaction::new("from", "to", "1")
            .with_gas("21000")
            .with_gas_price("100")
            .with_nonce("7")
            .with_data("0xdeadbeef");

        assert_eq!(tx.from, "from");
        assert_eq!(tx.to, "to");
        assert_eq!(tx.value, "1");
        assert_eq!(tx.gas, Some("21000".to_string()));
        assert_eq!(tx.gas_price, Some("100".to_string()));
        assert_eq!(tx.nonce, Some("7".to_string()));
        assert_eq!(tx.data, Some("0xdeadbeef".to_string()));
    }

    #[test]
    fn test_transaction_validate_errors() {
        let tx = Transaction::new("", "to", "1");
        assert_eq!(tx.validate().unwrap_err(), "From address cannot be empty");

        let tx = Transaction::new("from", "", "1");
        assert_eq!(tx.validate().unwrap_err(), "To address cannot be empty");

        let tx = Transaction::new("from", "to", "");
        assert_eq!(tx.validate().unwrap_err(), "Value cannot be empty");
    }

    #[test]
    fn test_transaction_validate_ok() {
        let tx = Transaction::new("from", "to", "1");
        assert!(tx.validate().is_ok());
    }
}
