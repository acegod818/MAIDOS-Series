# maidos-config

> TOML 配置管理庫 | 環境變數展開 | 熱重載

[![crates.io](https://img.shields.io/crates/v/maidos-config.svg)](https://crates.io/crates/maidos-config)
[![docs.rs](https://docs.rs/maidos-config/badge.svg)](https://docs.rs/maidos-config)

## 功能

- ✅ TOML 配置解析
- ✅ 環境變數展開 (`${VAR}`, `${VAR:-default}`)
- ✅ Schema 驗證
- ✅ 熱重載 (File Watch)
- ✅ 執行緒安全 (Arc + RwLock)
- ✅ C FFI 支援

## 使用

```rust
use maidos_config::MaidosConfig;
use std::path::Path;

// 從檔案載入
let config = MaidosConfig::load(Path::new("config.toml"))?;

// 存取配置
println!("Provider: {}", config.llm().default_provider);
println!("Budget: ${}", config.llm().budget_daily);

// 熱重載
let handle = config.watch(|new_config| {
    println!("Config reloaded!");
})?;
```

## 配置格式

```toml
[maidos]
version = "1.0"

[llm]
default_provider = "anthropic"
budget_daily = 10.0
budget_monthly = 100.0

[llm.providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
model = "claude-sonnet-4-20250514"

[bus]
endpoint = "tcp://127.0.0.1:5555"

[auth]
token_ttl = 3600
secret = "${AUTH_SECRET:-default_secret}"
```

## FFI

```c
MaidosConfig* config = maidos_config_load("config.toml");
const char* value = maidos_config_get_string(config, "llm.default_provider");
maidos_config_free(config);
```

## License

MIT
