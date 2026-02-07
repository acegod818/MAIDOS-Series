# maidos-shared -- Configuration Reference

| Field     | Value                     |
|-----------|---------------------------|
| Product   | maidos-shared              |
| Version   | 0.2.0                     |
| Type      | Configuration Guide        |

## Overview

Configuration in the MAIDOS shared workspace is managed primarily through `maidos-config`, which provides TOML-based configuration loading, environment variable expansion, schema validation, and optional hot-reload via file watching.

Other crates consume configuration through typed structs deserialized by `maidos-config`.

## maidos-config

### Configuration File Format

The primary configuration file uses TOML format:

```toml
[general]
app_name = "my-maidos-app"
log_level = "info"

[auth]
secret = "${MAIDOS_AUTH_SECRET}"
token_ttl_secs = 3600

[llm]
default_provider = "openai"
api_key = "${OPENAI_API_KEY}"
```

### Environment Variable Expansion

Two syntaxes are supported:

| Syntax               | Behavior                                |
|----------------------|-----------------------------------------|
| `${VAR}`             | Substitute value; error if unset        |
| `${VAR:-default}`    | Substitute value; use default if unset  |

### Hot-Reload (watcher feature)

When the `watcher` feature is enabled (default), `maidos-config` monitors the configuration file for changes and triggers a reload callback. Disable with `--no-default-features`.

### Cargo Feature Flags

| Feature   | Default | Description                   |
|-----------|---------|-------------------------------|
| `watcher` | Yes     | Enable file watching via notify |

## maidos-auth

| Key                   | Type    | Default | Description                       |
|-----------------------|---------|---------|-----------------------------------|
| `auth.secret`         | string  | -       | HMAC-SHA256 signing secret (required) |
| `auth.token_ttl_secs` | u64    | 3600    | Token time-to-live in seconds     |

The secret should be provided via environment variable to avoid committing it to version control.

## maidos-bus

| Key                    | Type    | Default      | Description                      |
|------------------------|---------|--------------|----------------------------------|
| `bus.bind_address`     | string  | "127.0.0.1"  | TCP bind address                 |
| `bus.port`             | u16     | 9100         | TCP port for event bus           |
| `bus.channel_capacity` | usize   | 1024         | Bounded channel capacity         |

## maidos-llm

| Key                       | Type    | Default     | Description                        |
|---------------------------|---------|-------------|------------------------------------|
| `llm.default_provider`    | string  | "ollama"    | Default LLM provider               |
| `llm.api_key`             | string  | -           | API key for cloud providers        |
| `llm.endpoint`            | string  | -           | Custom endpoint URL                |
| `llm.timeout_secs`        | u64     | 30          | Request timeout in seconds         |
| `llm.max_retries`         | u32     | 3           | Maximum retry attempts             |

### Provider-Specific Configuration

```toml
[llm.openai]
api_key = "${OPENAI_API_KEY}"
model = "gpt-4o"

[llm.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
model = "claude-3-sonnet-20240229"

[llm.ollama]
endpoint = "http://localhost:11434"
model = "llama3"

[llm.azure]
api_key = "${AZURE_OPENAI_KEY}"
endpoint = "https://your-resource.openai.azure.com"
deployment = "gpt-4o"
api_version = "2024-02-15-preview"
```

## maidos-log

| Key              | Type    | Default  | Description                             |
|------------------|---------|----------|-----------------------------------------|
| `log.level`      | string  | "info"   | Log level (trace/debug/info/warn/error) |
| `log.format`     | string  | "json"   | Output format (json / pretty)           |
| `log.output`     | string  | "stdout" | Output sink (stdout / file path)        |

## maidos-social

| Key                          | Type    | Default | Description                    |
|------------------------------|---------|---------|--------------------------------|
| `social.twitter.api_key`    | string  | -       | Twitter API key                |
| `social.discord.bot_token`  | string  | -       | Discord bot token              |
| `social.telegram.bot_token` | string  | -       | Telegram bot token             |
| `social.rate_limit_ms`      | u64     | 1000    | Minimum interval between calls |

## maidos-google

| Key                              | Type    | Default | Description                   |
|----------------------------------|---------|---------|-------------------------------|
| `google.client_id`               | string  | -       | OAuth2 client ID              |
| `google.client_secret`           | string  | -       | OAuth2 client secret          |
| `google.service_account_key`     | string  | -       | Path to service account JSON  |

## maidos-p2p

| Key                      | Type    | Default    | Description                          |
|--------------------------|---------|------------|--------------------------------------|
| `p2p.listen_address`    | string  | "0.0.0.0"  | Listen address for peer connections  |
| `p2p.listen_port`       | u16     | 9200       | Listen port                          |
| `p2p.bootstrap_peers`   | array   | []         | Initial peer addresses               |
| `p2p.max_peers`         | usize   | 50         | Maximum peer connections             |

## maidos-chain

| Key                      | Type    | Default                      | Description                 |
|--------------------------|---------|------------------------------|-----------------------------|
| `chain.rpc_url`          | string  | "http://localhost:8545"      | JSON-RPC endpoint           |
| `chain.chain_id`         | u64     | 1                            | Ethereum chain ID           |
| `chain.wallet_path`      | string  | -                            | Path to keystore file       |

## Precedence

Configuration values are resolved in this order (highest precedence first):

1. Environment variables
2. Configuration file values
3. Hard-coded defaults

*maidos-shared CONFIG v0.2.0 -- CodeQC Gate C Compliant*
