//! MAIDOS Unified Tool Format
//!
//! This module provides a unified tool/function calling format that can be
//! converted to/from various LLM provider formats.
//!
//! # Example
//! ```ignore
//! use maidos_llm::tool::{MaidosTool, ToolParameter};
//!
//! let tool = MaidosTool::new("get_weather", "Get weather for a location")
//!     .parameter(ToolParameter::new("location", "string")
//!         .description("City name")
//!         .required(true))
//!     .parameter(ToolParameter::new("unit", "string")
//!         .description("Temperature unit")
//!         .enum_values(vec!["celsius", "fahrenheit"]));
//!
//! // Convert to OpenAI format
//! let openai_tool = tool.to_openai();
//!
//! // Convert to Anthropic format
//! let anthropic_tool = tool.to_anthropic();
//! ```

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// MAIDOS unified tool format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaidosTool {
    /// Tool name (function name)
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Tool parameters (JSON Schema format)
    pub parameters: ToolParameters,

    /// Provider-specific hints
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_hints: Option<ProviderHints>,
}

impl MaidosTool {
    /// Create a new tool
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: ToolParameters::default(),
            provider_hints: None,
        }
    }

    /// Add a parameter
    pub fn parameter(mut self, param: ToolParameter) -> Self {
        let required = param.required;
        let name = param.name.clone();

        self.parameters.properties.insert(name.clone(), param);

        if required {
            self.parameters.required.push(name);
        }

        self
    }

    /// Set provider hints
    pub fn hints(mut self, hints: ProviderHints) -> Self {
        self.provider_hints = Some(hints);
        self
    }

    /// Check if tool has any parameters
    pub fn has_parameters(&self) -> bool {
        !self.parameters.properties.is_empty()
    }

    /// Get parameter count
    pub fn parameter_count(&self) -> usize {
        self.parameters.properties.len()
    }
}

/// Tool parameters in JSON Schema format
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolParameters {
    /// Parameter type (always "object")
    #[serde(rename = "type", default = "default_object_type")]
    pub param_type: String,

    /// Property definitions
    #[serde(default)]
    pub properties: HashMap<String, ToolParameter>,

    /// Required parameter names
    #[serde(default)]
    pub required: Vec<String>,

    /// Allow additional properties
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<bool>,
}

fn default_object_type() -> String {
    "object".to_string()
}

/// A single tool parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    /// Parameter name
    #[serde(skip)]
    pub name: String,

    /// Parameter type ("string", "number", "integer", "boolean", "array", "object")
    #[serde(rename = "type")]
    pub param_type: String,

    /// Human-readable description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Is this parameter required
    #[serde(skip)]
    pub required: bool,

    /// Default value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,

    /// Allowed enum values
    #[serde(default, rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,

    /// For array type: item schema
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ToolParameter>>,

    /// For object type: nested properties
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, ToolParameter>>,

    /// Minimum value (for number/integer)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,

    /// Maximum value (for number/integer)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,

    /// Minimum length (for string/array)
    #[serde(default, rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u32>,

    /// Maximum length (for string/array)
    #[serde(default, rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,

    /// Pattern (regex for string)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

impl ToolParameter {
    /// Create a new parameter
    pub fn new(name: impl Into<String>, param_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param_type: param_type.into(),
            description: None,
            required: false,
            default: None,
            enum_values: None,
            items: None,
            properties: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            pattern: None,
        }
    }

    /// Create a string parameter
    pub fn string(name: impl Into<String>) -> Self {
        Self::new(name, "string")
    }

    /// Create a number parameter
    pub fn number(name: impl Into<String>) -> Self {
        Self::new(name, "number")
    }

    /// Create an integer parameter
    pub fn integer(name: impl Into<String>) -> Self {
        Self::new(name, "integer")
    }

    /// Create a boolean parameter
    pub fn boolean(name: impl Into<String>) -> Self {
        Self::new(name, "boolean")
    }

    /// Create an array parameter
    pub fn array(name: impl Into<String>, items: ToolParameter) -> Self {
        Self {
            items: Some(Box::new(items)),
            ..Self::new(name, "array")
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Mark as required
    pub fn required(mut self, req: bool) -> Self {
        self.required = req;
        self
    }

    /// Set default value
    pub fn default_value(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }

    /// Set enum values
    pub fn enum_values(mut self, values: Vec<impl Into<String>>) -> Self {
        self.enum_values = Some(values.into_iter().map(Into::into).collect());
        self
    }

    /// Set minimum value
    pub fn min(mut self, val: f64) -> Self {
        self.minimum = Some(val);
        self
    }

    /// Set maximum value
    pub fn max(mut self, val: f64) -> Self {
        self.maximum = Some(val);
        self
    }

    /// Set min/max length
    pub fn length_range(mut self, min: u32, max: u32) -> Self {
        self.min_length = Some(min);
        self.max_length = Some(max);
        self
    }

    /// Set pattern
    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }
}

/// Provider-specific hints
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderHints {
    /// Anthropic-specific hints
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anthropic: Option<AnthropicHints>,

    /// Google-specific hints
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub google: Option<GoogleHints>,

    /// OpenAI-specific hints
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openai: Option<OpenAiHints>,
}

/// Anthropic-specific hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicHints {
    /// Cache control for tool definition
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Cache control for Anthropic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub cache_type: String,
}

impl CacheControl {
    /// Create ephemeral cache control
    pub fn ephemeral() -> Self {
        Self {
            cache_type: "ephemeral".to_string(),
        }
    }
}

/// Google-specific hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleHints {
    /// Function calling mode
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

/// OpenAI-specific hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiHints {
    /// Strict mode for function calling
    #[serde(default)]
    pub strict: bool,
}

/// Trait for converting to provider formats
pub trait ToProviderFormat {
    /// Convert to OpenAI tool format
    fn to_openai(&self) -> Value;

    /// Convert to Anthropic tool format
    fn to_anthropic(&self) -> Value;

    /// Convert to Google tool format
    fn to_google(&self) -> Value;

    /// Convert to Mistral tool format
    fn to_mistral(&self) -> Value;

    /// Convert to Cohere tool format
    fn to_cohere(&self) -> Value;
}

impl ToProviderFormat for MaidosTool {
    fn to_openai(&self) -> Value {
        let strict = self
            .provider_hints
            .as_ref()
            .and_then(|h| h.openai.as_ref())
            .map(|o| o.strict)
            .unwrap_or(false);

        json!({
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": {
                    "type": "object",
                    "properties": self.parameters.properties.iter()
                        .map(|(k, v)| (k.clone(), param_to_json_schema(v)))
                        .collect::<HashMap<_, _>>(),
                    "required": self.parameters.required,
                },
                "strict": strict,
            }
        })
    }

    fn to_anthropic(&self) -> Value {
        let mut tool = json!({
            "name": self.name,
            "description": self.description,
            "input_schema": {
                "type": "object",
                "properties": self.parameters.properties.iter()
                    .map(|(k, v)| (k.clone(), param_to_json_schema(v)))
                    .collect::<HashMap<_, _>>(),
                "required": self.parameters.required,
            }
        });

        // Add cache control if specified
        if let Some(hints) = &self.provider_hints {
            if let Some(anthropic) = &hints.anthropic {
                if let Some(cache) = &anthropic.cache_control {
                    tool["cache_control"] = json!(cache);
                }
            }
        }

        tool
    }

    fn to_google(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "parameters": {
                "type": "object",
                "properties": self.parameters.properties.iter()
                    .map(|(k, v)| (k.clone(), param_to_json_schema(v)))
                    .collect::<HashMap<_, _>>(),
                "required": self.parameters.required,
            }
        })
    }

    fn to_mistral(&self) -> Value {
        // Mistral uses OpenAI-compatible format
        self.to_openai()
    }

    fn to_cohere(&self) -> Value {
        // Cohere has a slightly different format
        json!({
            "name": self.name,
            "description": self.description,
            "parameter_definitions": self.parameters.properties.iter()
                .map(|(k, v)| (k.clone(), json!({
                    "type": v.param_type,
                    "description": v.description.clone().unwrap_or_default(),
                    "required": self.parameters.required.contains(k),
                })))
                .collect::<HashMap<_, _>>(),
        })
    }
}

/// Convert ToolParameter to JSON Schema
fn param_to_json_schema(param: &ToolParameter) -> Value {
    let mut schema = json!({
        "type": param.param_type,
    });

    if let Some(desc) = &param.description {
        schema["description"] = json!(desc);
    }

    if let Some(default) = &param.default {
        schema["default"] = default.clone();
    }

    if let Some(enums) = &param.enum_values {
        schema["enum"] = json!(enums);
    }

    if let Some(items) = &param.items {
        schema["items"] = param_to_json_schema(items);
    }

    if let Some(props) = &param.properties {
        schema["properties"] = json!(props
            .iter()
            .map(|(k, v)| (k.clone(), param_to_json_schema(v)))
            .collect::<HashMap<_, _>>());
    }

    if let Some(min) = param.minimum {
        schema["minimum"] = json!(min);
    }

    if let Some(max) = param.maximum {
        schema["maximum"] = json!(max);
    }

    if let Some(min_len) = param.min_length {
        schema["minLength"] = json!(min_len);
    }

    if let Some(max_len) = param.max_length {
        schema["maxLength"] = json!(max_len);
    }

    if let Some(pattern) = &param.pattern {
        schema["pattern"] = json!(pattern);
    }

    schema
}

/// Tool call result from LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID (for matching responses)
    pub id: String,

    /// Tool name
    pub name: String,

    /// Arguments as JSON
    pub arguments: Value,
}

impl ToolCall {
    /// Create a new tool call
    pub fn new(id: impl Into<String>, name: impl Into<String>, arguments: Value) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            arguments,
        }
    }

    /// Parse arguments as a specific type
    pub fn parse_arguments<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.arguments.clone())
    }
}

/// Tool execution result to send back to LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool call ID (must match the call)
    pub tool_call_id: String,

    /// Result content
    pub content: String,

    /// Is this an error result
    #[serde(default)]
    pub is_error: bool,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
            is_error: false,
        }
    }

    /// Create an error result
    pub fn error(tool_call_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: error.into(),
            is_error: true,
        }
    }
}

/// Parse tool call from OpenAI format
pub fn parse_openai_tool_call(tool_call: &Value) -> Option<ToolCall> {
    let id = tool_call.get("id")?.as_str()?;
    let function = tool_call.get("function")?;
    let name = function.get("name")?.as_str()?;
    let arguments_str = function.get("arguments")?.as_str()?;
    let arguments = serde_json::from_str(arguments_str).ok()?;

    Some(ToolCall::new(id, name, arguments))
}

/// Parse tool call from Anthropic format
pub fn parse_anthropic_tool_call(content_block: &Value) -> Option<ToolCall> {
    let id = content_block.get("id")?.as_str()?;
    let name = content_block.get("name")?.as_str()?;
    let input = content_block.get("input")?;

    Some(ToolCall::new(id, name, input.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_creation() {
        let tool = MaidosTool::new("get_weather", "Get weather for a location");
        assert_eq!(tool.name, "get_weather");
        assert_eq!(tool.description, "Get weather for a location");
        assert!(!tool.has_parameters());
    }

    #[test]
    fn test_tool_with_parameters() {
        let tool = MaidosTool::new("get_weather", "Get weather")
            .parameter(
                ToolParameter::string("location")
                    .description("City name")
                    .required(true),
            )
            .parameter(
                ToolParameter::string("unit")
                    .description("Temperature unit")
                    .enum_values(vec!["celsius", "fahrenheit"]),
            );

        assert!(tool.has_parameters());
        assert_eq!(tool.parameter_count(), 2);
        assert_eq!(tool.parameters.required, vec!["location"]);
    }

    #[test]
    fn test_parameter_types() {
        let string_param = ToolParameter::string("name");
        assert_eq!(string_param.param_type, "string");

        let number_param = ToolParameter::number("price");
        assert_eq!(number_param.param_type, "number");

        let int_param = ToolParameter::integer("count");
        assert_eq!(int_param.param_type, "integer");

        let bool_param = ToolParameter::boolean("active");
        assert_eq!(bool_param.param_type, "boolean");
    }

    #[test]
    fn test_array_parameter() {
        let items = ToolParameter::string("item");
        let array_param = ToolParameter::array("tags", items);

        assert_eq!(array_param.param_type, "array");
        assert!(array_param.items.is_some());
    }

    #[test]
    fn test_parameter_constraints() {
        let param = ToolParameter::number("temperature")
            .min(-50.0)
            .max(50.0);

        assert_eq!(param.minimum, Some(-50.0));
        assert_eq!(param.maximum, Some(50.0));
    }

    #[test]
    fn test_string_constraints() {
        let param = ToolParameter::string("code")
            .length_range(3, 10)
            .pattern(r"^[A-Z]+$");

        assert_eq!(param.min_length, Some(3));
        assert_eq!(param.max_length, Some(10));
        assert_eq!(param.pattern, Some(r"^[A-Z]+$".to_string()));
    }

    #[test]
    fn test_to_openai_format() {
        let tool = MaidosTool::new("test", "Test tool")
            .parameter(ToolParameter::string("arg").required(true));

        let openai = tool.to_openai();

        assert_eq!(openai["type"], "function");
        assert_eq!(openai["function"]["name"], "test");
        assert_eq!(openai["function"]["description"], "Test tool");
    }

    #[test]
    fn test_to_anthropic_format() {
        let tool = MaidosTool::new("test", "Test tool")
            .parameter(ToolParameter::string("arg").required(true));

        let anthropic = tool.to_anthropic();

        assert_eq!(anthropic["name"], "test");
        assert_eq!(anthropic["description"], "Test tool");
        assert!(anthropic.get("input_schema").is_some());
    }

    #[test]
    fn test_to_google_format() {
        let tool = MaidosTool::new("test", "Test tool")
            .parameter(ToolParameter::string("arg").required(true));

        let google = tool.to_google();

        assert_eq!(google["name"], "test");
        assert!(google.get("parameters").is_some());
    }

    #[test]
    fn test_to_cohere_format() {
        let tool = MaidosTool::new("test", "Test tool")
            .parameter(ToolParameter::string("arg").required(true));

        let cohere = tool.to_cohere();

        assert_eq!(cohere["name"], "test");
        assert!(cohere.get("parameter_definitions").is_some());
    }

    #[test]
    fn test_to_mistral_format() {
        let tool = MaidosTool::new("test", "Test tool");
        let mistral = tool.to_mistral();

        // Mistral uses OpenAI format
        assert_eq!(mistral["type"], "function");
    }

    #[test]
    fn test_anthropic_cache_hints() {
        let tool = MaidosTool::new("test", "Test")
            .hints(ProviderHints {
                anthropic: Some(AnthropicHints {
                    cache_control: Some(CacheControl::ephemeral()),
                }),
                ..Default::default()
            });

        let anthropic = tool.to_anthropic();
        assert!(anthropic.get("cache_control").is_some());
    }

    #[test]
    fn test_openai_strict_mode() {
        let tool = MaidosTool::new("test", "Test")
            .hints(ProviderHints {
                openai: Some(OpenAiHints { strict: true }),
                ..Default::default()
            });

        let openai = tool.to_openai();
        assert_eq!(openai["function"]["strict"], true);
    }

    #[test]
    fn test_tool_call_creation() {
        let call = ToolCall::new("call_123", "get_weather", json!({"location": "NYC"}));

        assert_eq!(call.id, "call_123");
        assert_eq!(call.name, "get_weather");
        assert_eq!(call.arguments["location"], "NYC");
    }

    #[test]
    fn test_tool_call_parse_arguments() {
        #[derive(Deserialize)]
        struct WeatherArgs {
            location: String,
        }

        let call = ToolCall::new("call_123", "get_weather", json!({"location": "NYC"}));
        let args: WeatherArgs = call.parse_arguments().expect("parse failed");

        assert_eq!(args.location, "NYC");
    }

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success("call_123", "Temperature: 72°F");

        assert_eq!(result.tool_call_id, "call_123");
        assert!(!result.is_error);
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("call_123", "Location not found");

        assert_eq!(result.tool_call_id, "call_123");
        assert!(result.is_error);
    }

    #[test]
    fn test_parse_openai_tool_call() {
        let openai_call = json!({
            "id": "call_abc123",
            "type": "function",
            "function": {
                "name": "get_weather",
                "arguments": "{\"location\":\"NYC\"}"
            }
        });

        let call = parse_openai_tool_call(&openai_call).expect("parse failed");
        assert_eq!(call.id, "call_abc123");
        assert_eq!(call.name, "get_weather");
    }

    #[test]
    fn test_parse_anthropic_tool_call() {
        let anthropic_call = json!({
            "type": "tool_use",
            "id": "toolu_123",
            "name": "get_weather",
            "input": {"location": "NYC"}
        });

        let call = parse_anthropic_tool_call(&anthropic_call).expect("parse failed");
        assert_eq!(call.id, "toolu_123");
        assert_eq!(call.name, "get_weather");
    }

    #[test]
    fn test_complex_tool() {
        let tool = MaidosTool::new("search_flights", "Search for flights")
            .parameter(
                ToolParameter::string("origin")
                    .description("Origin airport code")
                    .required(true)
                    .length_range(3, 3)
                    .pattern(r"^[A-Z]{3}$"),
            )
            .parameter(
                ToolParameter::string("destination")
                    .description("Destination airport code")
                    .required(true),
            )
            .parameter(
                ToolParameter::string("date")
                    .description("Travel date (YYYY-MM-DD)")
                    .required(true),
            )
            .parameter(
                ToolParameter::integer("passengers")
                    .description("Number of passengers")
                    .default_value(json!(1))
                    .min(1.0)
                    .max(9.0),
            )
            .parameter(
                ToolParameter::string("class")
                    .description("Travel class")
                    .enum_values(vec!["economy", "business", "first"]),
            );

        assert_eq!(tool.parameter_count(), 5);
        assert_eq!(tool.parameters.required.len(), 3);

        // Test all formats work
        let _ = tool.to_openai();
        let _ = tool.to_anthropic();
        let _ = tool.to_google();
        let _ = tool.to_cohere();
        let _ = tool.to_mistral();
    }
}
