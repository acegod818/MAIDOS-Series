# maidos-shared — Public API Contracts

## 1. Contract Principles

- All public types are `Send + Sync` unless documented otherwise.
- All fallible operations return `Result<T, CrateError>`.
- Breaking changes require a major version bump.
- Deprecated items are marked `#[deprecated(since, note)]` for one minor cycle before removal.

---

## 2. maidos-auth

```rust
pub struct AuthClient { /* private */ }
pub struct AuthConfig { pub provider: AuthProvider, pub client_id: String, pub redirect_uri: String }
pub struct AuthToken  { pub access_token: String, pub refresh_token: Option<String>, pub expires_at: DateTime<Utc> }
pub enum   AuthError  { InvalidCredentials, TokenExpired, NetworkError(reqwest::Error), ConfigError(ConfigError) }

impl AuthClient {
    pub fn new(config: AuthConfig) -> Result<Self, AuthError>;
    pub async fn authenticate(&self) -> Result<AuthToken, AuthError>;
    pub async fn ensure_valid_token(&self) -> Result<AuthToken, AuthError>;
    pub async fn revoke(&self) -> Result<(), AuthError>;
}
```

## 3. maidos-bus

```rust
pub struct MessageBus { /* private */ }
pub struct Subscription { pub id: Uuid, pub topic: String }
pub enum   BusError { TopicNotFound, SendFailed, Closed }

impl MessageBus {
    pub fn new() -> Self;
    pub fn subscribe<F>(&self, topic: &str, handler: F) -> Subscription where F: Fn(&[u8]) + Send + Sync + 'static;
    pub fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), BusError>;
    pub async fn close(&self) -> Result<(), BusError>;
}
```

## 4. maidos-config

```rust
pub struct Config { /* private */ }
pub struct ConfigBuilder { /* private */ }
pub enum   ConfigError { MissingKey(String), ParseError(String), IoError(std::io::Error) }

impl ConfigBuilder {
    pub fn new() -> Self;
    pub fn file(self, path: &Path) -> Self;
    pub fn env_prefix(self, prefix: &str) -> Self;
    pub fn build(self) -> Result<Config, ConfigError>;
}
impl Config {
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, ConfigError>;
    pub fn on_change<F>(&self, callback: F) where F: Fn(&Config) + Send + 'static;
}
```

## 5. maidos-llm

```rust
pub struct OllamaClient     { /* private */ }
pub struct GenerateRequest   { pub model: String, pub prompt: String, pub options: Option<GenerateOptions> }
pub struct GenerateResponse  { pub text: String, pub done: bool }
pub enum   LlmError          { ConnectionRefused, ModelNotFound(String), Timeout, ServerError(String) }

impl OllamaClient {
    pub fn new(endpoint: &str) -> Result<Self, LlmError>;
    pub async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse, LlmError>;
    pub async fn generate_stream(&self, request: GenerateRequest) -> impl Stream<Item = Result<GenerateResponse, LlmError>>;
    pub async fn list_models(&self) -> Result<Vec<String>, LlmError>;
}
```

## 6. maidos-log

```rust
pub struct LogConfig { pub level: LogLevel, pub format: LogFormat, pub output: LogOutput }
pub enum   LogLevel  { Trace, Debug, Info, Warn, Error }
pub enum   LogFormat { Json, Pretty }
pub enum   LogOutput { Stdout, File(PathBuf), Both(PathBuf) }

pub fn init(config: LogConfig) -> Result<(), Box<dyn std::error::Error>>;
pub fn set_level(level: LogLevel);
```

## 7. Stability Guarantee

Public APIs listed above are considered stable. Internal modules prefixed with `_internal`
or marked `#[doc(hidden)]` are not part of the contract.
