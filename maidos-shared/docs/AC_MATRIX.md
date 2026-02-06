# maidos-shared — Acceptance Criteria Matrix

## Overview

Each acceptance criterion is tied to a sub-crate and verifiable by automated test.

## maidos-auth

| ID | Criterion | Verification |
|----|-----------|-------------|
| AC-AUTH-01 | `AuthClient::new` returns `Ok` with valid config | Unit test |
| AC-AUTH-02 | `authenticate()` returns a valid `AuthToken` on success | Integration test |
| AC-AUTH-03 | Expired tokens are auto-refreshed before API calls | Unit test with mocked clock |
| AC-AUTH-04 | Invalid credentials return `AuthError::InvalidCredentials` | Unit test |
| AC-AUTH-05 | Token cache persists across restarts | Integration test |

## maidos-bus

| ID | Criterion | Verification |
|----|-----------|-------------|
| AC-BUS-01 | `publish` delivers to all subscribers of a topic | Unit test |
| AC-BUS-02 | Messages maintain ordering per topic | Unit test |
| AC-BUS-03 | Unsubscribed handlers stop receiving messages | Unit test |
| AC-BUS-04 | Bus gracefully drains on `close()` | Unit test with pending messages |

## maidos-chain

| ID | Criterion | Verification |
|----|-----------|-------------|
| AC-CHN-01 | Audit entries are appended with valid hash chain | Unit test |
| AC-CHN-02 | Tampered entries are detected on verification | Unit test |

## maidos-config

| ID | Criterion | Verification |
|----|-----------|-------------|
| AC-CFG-01 | Config loads from TOML file | Unit test |
| AC-CFG-02 | Environment variables override file values | Unit test |
| AC-CFG-03 | Missing required keys return `ConfigError::MissingKey` | Unit test |
| AC-CFG-04 | Config hot-reload triggers change notification | Integration test |

## maidos-google

| ID | Criterion | Verification |
|----|-----------|-------------|
| AC-GOO-01 | Google API client authenticates with service account | Integration test |
| AC-GOO-02 | API errors map to typed `GoogleError` variants | Unit test |

## maidos-llm

| ID | Criterion | Verification |
|----|-----------|-------------|
| AC-LLM-01 | `OllamaClient` connects to local Ollama instance | Integration test |
| AC-LLM-02 | `generate()` returns model response within timeout | Integration test |
| AC-LLM-03 | Connection failure returns `LlmError::ConnectionRefused` | Unit test |
| AC-LLM-04 | Unknown model returns `LlmError::ModelNotFound` | Unit test |

## maidos-log

| ID | Criterion | Verification |
|----|-----------|-------------|
| AC-LOG-01 | `init()` configures global tracing subscriber | Unit test |
| AC-LOG-02 | Log output matches configured format (JSON/text) | Unit test |
| AC-LOG-03 | Runtime level change takes effect without restart | Integration test |

## maidos-p2p

| ID | Criterion | Verification |
|----|-----------|-------------|
| AC-P2P-01 | Two nodes discover each other on local network | Integration test |
| AC-P2P-02 | Messages are delivered between connected peers | Integration test |
| AC-P2P-03 | Disconnected peer triggers `PeerDisconnected` event | Unit test |

## maidos-social

| ID | Criterion | Verification |
|----|-----------|-------------|
| AC-SOC-01 | Social client posts to configured platform | Integration test |
| AC-SOC-02 | Rate limit errors are retried with backoff | Unit test |
| AC-SOC-03 | Invalid API keys return `SocialError::Unauthorized` | Unit test |
