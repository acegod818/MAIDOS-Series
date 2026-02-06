# maidos-shared — Product Requirements Document

## 1. Overview

maidos-shared is the foundational shared library crate for the MAIDOS ecosystem. It provides
common infrastructure consumed by all MAIDOS products including MAIDOS-Driver, MAIDOS-IME,
MAIDOS-Forge, and MAIDOS-CodeQC.

## 2. Problem Statement

Each MAIDOS product requires identical cross-cutting capabilities: authentication, logging,
configuration, messaging, and external service integration. Duplicating this logic across
products leads to inconsistency, increased maintenance burden, and divergent behavior.

## 3. Goals

| ID | Goal | Success Metric |
|----|------|----------------|
| G-1 | Single source of truth for shared logic | Zero duplicated utility code across products |
| G-2 | Feature-gated modularity | Each sub-crate independently selectable via Cargo features |
| G-3 | Stable public API surface | No unplanned breaking changes between minor versions |
| G-4 | Cross-product compatibility | All 5+ MAIDOS products compile against the same version |

## 4. Sub-Crate Inventory

| Sub-Crate | Responsibility |
|-----------|---------------|
| maidos-auth | OAuth2 / token-based authentication |
| maidos-bus | Intra-process and inter-process message bus |
| maidos-chain | Blockchain integration for audit trails |
| maidos-config | Hierarchical configuration management |
| maidos-google | Google API client wrappers |
| maidos-llm | LLM integration via Ollama |
| maidos-log | Structured logging with tracing backend |
| maidos-p2p | Peer-to-peer networking layer |
| maidos-social | Social media platform integration |

## 5. Target Consumers

- **MAIDOS-Driver** — Windows driver management tool (Rust cdylib + C# WPF)
- **MAIDOS-IME** — Input method engine
- **MAIDOS-Forge** — Build and packaging toolchain
- **MAIDOS-CodeQC** — Code quality gate system

## 6. Out of Scope

- Product-specific business logic (belongs in each product crate)
- GUI components (each product owns its own UI layer)
- Platform-specific FFI bindings (handled by consumer crates)

## 7. Release Cadence

maidos-shared follows semantic versioning. Patch releases are published as needed. Minor
releases align with MAIDOS product milestones. Major releases require an ADR and migration
guide.
