//! Configuration file loader with environment variable expansion
//!
//! <impl>
//! WHAT: Load TOML files, expand ${ENV_VAR} syntax, validate schema
//! WHY: Central config loading logic, reusable by all modules
//! HOW: Read file -> expand env vars -> parse TOML -> validate -> return typed struct
//! TEST: Valid file loads, missing file errors, env var expansion, invalid TOML errors
//! </impl>

use crate::error::{ConfigError, Result};
use crate::schema::MaidosConfigSchema;
use std::fs;
use std::path::Path;
use tracing::{debug, instrument};

/// Load configuration from a TOML file
///
/// # Arguments
/// * `path` - Path to the TOML configuration file
///
/// # Returns
/// * `Ok(MaidosConfigSchema)` - Parsed and validated configuration
/// * `Err(ConfigError)` - If file not found, parse error, or validation fails
///
/// # Example
/// ```ignore
/// let config = load_config(Path::new("maidos.toml"))?;
/// println!("Default provider: {}", config.llm.default_provider);
/// ```
#[instrument(skip_all, fields(path = %path.display()))]
pub fn load_config(path: &Path) -> Result<MaidosConfigSchema> {
    // Check file exists
    if !path.exists() {
        return Err(ConfigError::FileNotFound(path.to_path_buf()));
    }

    // Read file contents
    debug!("Reading config file");
    let contents = fs::read_to_string(path)?;

    // Expand environment variables
    debug!("Expanding environment variables");
    let expanded = expand_env_vars(&contents)?;

    // Parse TOML
    debug!("Parsing TOML");
    let schema: MaidosConfigSchema = toml::from_str(&expanded)?;

    // Validate schema
    debug!("Validating schema");
    validate_schema(&schema)?;

    debug!("Config loaded successfully");
    Ok(schema)
}

/// Load configuration from a TOML string (for testing)
pub fn load_config_str(contents: &str) -> Result<MaidosConfigSchema> {
    let expanded = expand_env_vars(contents)?;
    let schema: MaidosConfigSchema = toml::from_str(&expanded)?;
    validate_schema(&schema)?;
    Ok(schema)
}

/// Expand ${ENV_VAR} syntax in string
///
/// Supports:
/// - ${VAR} - Required, errors if not set
/// - ${VAR:-default} - Optional with default value
fn expand_env_vars(input: &str) -> Result<String> {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'

            // Read variable name and optional default
            let mut var_expr = String::new();
            let mut depth = 1;

            for c in chars.by_ref() {
                if c == '{' {
                    depth += 1;
                    var_expr.push(c);
                } else if c == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    var_expr.push(c);
                } else {
                    var_expr.push(c);
                }
            }

            // Parse VAR or VAR:-default
            let (var_name, default_value) = if let Some(pos) = var_expr.find(":-") {
                (&var_expr[..pos], Some(&var_expr[pos + 2..]))
            } else {
                (var_expr.as_str(), None)
            };

            // Get environment variable
            match std::env::var(var_name) {
                Ok(value) => result.push_str(&value),
                Err(_) => {
                    if let Some(default) = default_value {
                        result.push_str(default);
                    } else {
                        return Err(ConfigError::EnvVarNotSet(var_name.to_string()));
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

/// Validate configuration schema
fn validate_schema(schema: &MaidosConfigSchema) -> Result<()> {
    // Validate version
    if schema.maidos.version.is_empty() {
        return Err(ConfigError::ValidationError(
            "maidos.version cannot be empty".to_string(),
        ));
    }

    // Validate budgets are non-negative
    if schema.llm.budget_daily < 0.0 {
        return Err(ConfigError::ValidationError(
            "llm.budget_daily must be non-negative".to_string(),
        ));
    }
    if schema.llm.budget_monthly < 0.0 {
        return Err(ConfigError::ValidationError(
            "llm.budget_monthly must be non-negative".to_string(),
        ));
    }

    // Validate daily <= monthly budget
    if schema.llm.budget_daily > schema.llm.budget_monthly {
        return Err(ConfigError::ValidationError(
            "llm.budget_daily cannot exceed llm.budget_monthly".to_string(),
        ));
    }

    // Validate bus endpoint format
    if !schema.bus.endpoint.starts_with("tcp://")
        && !schema.bus.endpoint.starts_with("ipc://")
        && !schema.bus.endpoint.starts_with("inproc://")
    {
        return Err(ConfigError::ValidationError(format!(
            "bus.endpoint must start with tcp://, ipc://, or inproc:// (got: {})",
            schema.bus.endpoint
        )));
    }

    // Validate auth token TTL
    if schema.auth.token_ttl == 0 {
        return Err(ConfigError::ValidationError(
            "auth.token_ttl must be greater than 0".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_expand_env_vars_simple() {
        env::set_var("TEST_VAR_1", "hello");
        let result = expand_env_vars("Value: ${TEST_VAR_1}").unwrap();
        assert_eq!(result, "Value: hello");
        env::remove_var("TEST_VAR_1");
    }

    #[test]
    fn test_expand_env_vars_with_default() {
        env::remove_var("NONEXISTENT_VAR");
        let result = expand_env_vars("Value: ${NONEXISTENT_VAR:-default_value}").unwrap();
        assert_eq!(result, "Value: default_value");
    }

    #[test]
    fn test_expand_env_vars_missing_required() {
        env::remove_var("REQUIRED_VAR");
        let result = expand_env_vars("Value: ${REQUIRED_VAR}");
        assert!(matches!(result, Err(ConfigError::EnvVarNotSet(_))));
    }

    #[test]
    fn test_expand_env_vars_no_expansion_needed() {
        let result = expand_env_vars("No variables here").unwrap();
        assert_eq!(result, "No variables here");
    }

    #[test]
    fn test_validate_schema_valid() {
        let schema = MaidosConfigSchema::default();
        assert!(validate_schema(&schema).is_ok());
    }

    #[test]
    fn test_validate_schema_negative_budget() {
        let mut schema = MaidosConfigSchema::default();
        schema.llm.budget_daily = -1.0;
        assert!(matches!(
            validate_schema(&schema),
            Err(ConfigError::ValidationError(_))
        ));
    }

    #[test]
    fn test_validate_schema_daily_exceeds_monthly() {
        let mut schema = MaidosConfigSchema::default();
        schema.llm.budget_daily = 200.0;
        schema.llm.budget_monthly = 100.0;
        assert!(matches!(
            validate_schema(&schema),
            Err(ConfigError::ValidationError(_))
        ));
    }

    #[test]
    fn test_validate_schema_invalid_endpoint() {
        let mut schema = MaidosConfigSchema::default();
        schema.bus.endpoint = "http://localhost:5555".to_string();
        assert!(matches!(
            validate_schema(&schema),
            Err(ConfigError::ValidationError(_))
        ));
    }

    #[test]
    fn test_load_config_str() {
        let toml = r#"
[maidos]
version = "1.0"

[llm]
default_provider = "test"
budget_daily = 5.0
budget_monthly = 50.0

[bus]
endpoint = "tcp://127.0.0.1:5555"

[auth]
token_ttl = 1800
"#;
        let schema = load_config_str(toml).unwrap();
        assert_eq!(schema.llm.default_provider, "test");
        assert_eq!(schema.llm.budget_daily, 5.0);
    }
}
