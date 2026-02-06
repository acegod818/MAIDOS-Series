# maidos-shared — Architecture Decision Records

---

## ADR-001: Monorepo Workspace for Shared Code

**Status:** Accepted
**Date:** 2026-01-15

### Context
MAIDOS products share authentication, logging, configuration, and messaging logic. We
needed to decide between (a) separate repositories per shared library, (b) a single
monorepo workspace, or (c) copy-paste shared code into each product.

### Decision
Adopt a Cargo workspace monorepo for all shared crates under `maidos-shared`.

### Rationale
- **Consistency:** A single repository guarantees all sub-crates compile together and
  share the same dependency versions via workspace-level `[dependencies]`.
- **Atomic changes:** Cross-crate refactors land in a single commit and single CI run.
- **Simplified CI:** One pipeline validates the entire shared surface.
- **Discoverability:** New developers find all shared code in one place.

### Consequences
- Repository size grows over time; mitigated by sparse checkouts if needed.
- Publishing to crates.io requires coordinated version bumps.

---

## ADR-002: Sub-Crate Modularity via Feature Flags

**Status:** Accepted
**Date:** 2026-01-15

### Context
Not every MAIDOS product needs every shared capability. MAIDOS-Driver needs auth, config,
and log but not social or chain. Compiling unused sub-crates increases build time and
binary size.

### Decision
Each sub-crate is an independent Cargo crate within the workspace. The root
`maidos-shared` crate re-exports them behind feature flags.

### Rationale
- **Compile-time savings:** Products enable only what they use.
- **Binary size reduction:** Unused code is excluded.
- **Independent versioning:** Sub-crates can advance at different paces.
- **Clear ownership:** Each sub-crate has a defined API boundary.

### Consequences
- Feature flag combinations must be tested in CI (feature matrix).
- Documentation must specify which features enable which modules.

---

## ADR-003: Ollama for LLM Integration (Privacy-First)

**Status:** Accepted
**Date:** 2026-01-20

### Context
Several MAIDOS products benefit from LLM capabilities (code review in CodeQC, driver
description parsing in Driver). Options considered: OpenAI API, Anthropic API, local
Ollama, llama.cpp direct.

### Decision
Use Ollama as the default LLM backend in `maidos-llm`.

### Rationale
- **Privacy:** All inference runs locally; no user data sent to third-party servers.
- **Cost:** Zero per-token cost after initial model download.
- **Flexibility:** Ollama supports multiple models (Llama, Mistral, Gemma) via a
  single HTTP API.
- **Simplicity:** REST API is straightforward; no native C++ binding needed.
- **Offline capability:** Works without internet once models are downloaded.

### Consequences
- Users must install Ollama separately (documented in USAGE.md).
- Inference quality depends on local hardware (GPU recommended).
- Future cloud LLM backends can be added as alternative feature flags.
