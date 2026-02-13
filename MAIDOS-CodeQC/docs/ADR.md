# Architecture Decision Records - MAIDOS-CodeQC

## Overview

This document records key architectural decisions made during the design and implementation of MAIDOS-CodeQC. Each decision is documented with context, options considered, and rationale.

---

## ADR-001: Language Choice - TypeScript over Python

**Date**: 2025-01-10
**Status**: Accepted
**Context**:
- MAIDOS-CodeQC needs to be fast, portable, and easy to integrate into CI/CD pipelines
- Target audience is primarily JavaScript/TypeScript developers
- Must support Tree-sitter for AST parsing
- Deployment via npm is preferred for Node.js ecosystem

**Options Considered**:
1. **Python** - Mature ecosystem, rich static analysis tools (AST module, pylint)
2. **TypeScript** - Fast, npm distribution, strong typing, web-tree-sitter support
3. **Rust** - Maximum performance, but complex toolchain and deployment

**Decision**: TypeScript

**Rationale**:
- Tree-sitter has first-class JavaScript bindings (`web-tree-sitter`)
- npm install workflow is frictionless for JS/TS developers
- TypeScript provides strong typing for maintainability
- Node.js 18+ is ubiquitous in CI environments
- ESM + CJS dual build support via `tsup`

**Consequences**:
- Positive: Fast iteration, easy CI integration, large contributor pool
- Negative: Memory usage higher than Rust (acceptable for CLI tool)

---

## ADR-002: Detection Strategy - Regex + AST, No LLM

**Date**: 2025-01-12
**Status**: Accepted
**Context**:
- Code-QC v3.5 requires deterministic, reproducible rule checking
- Zero external API dependencies for offline operation
- Must run in < 5 seconds for 10k LOC projects

**Options Considered**:
1. **LLM-based detection** - ChatGPT/Claude API for semantic analysis
2. **Pure regex** - Fast but limited to surface patterns
3. **Hybrid: Regex + Tree-sitter AST** - Balance speed and depth

**Decision**: Hybrid Regex + Tree-sitter AST

**Rationale**:
- **Regex** handles 70% of rules efficiently (R01, R02, R07, R10, etc.)
- **Tree-sitter AST** enables deep analysis for structural rules (P05, P06, P10)
- LLMs are too slow (> 1s per file), non-deterministic, and require network
- Zero-trust philosophy: all rules are local, no external calls

**Consequences**:
- Positive: 100% offline, reproducible, < 5s scan time
- Negative: Some semantic rules (R11, P01, P02) use heuristics, not true intent analysis

---

## ADR-003: Plugin System for Language Support

**Date**: 2025-01-15
**Status**: Accepted
**Context**:
- Code-QC v3.5 targets 97 languages (long-term roadmap)
- Phase 0 supports 5 core languages (TS/JS/Python/Rust/Go)
- Tree-sitter grammars are language-specific

**Options Considered**:
1. **Monolithic design** - Bundle all 97 Tree-sitter grammars
2. **Plugin architecture** - Optional peer dependencies per language
3. **Lazy loading** - Download grammars on-demand

**Decision**: Plugin architecture with peer dependencies

**Rationale**:
- npm package size would be > 100 MB with all grammars bundled
- Most projects use 1-3 languages, not 97
- `peerDependencies` allow users to install only needed grammars
- Plugin API enables community contributions for niche languages

**Consequences**:
- Positive: Small package size (< 1 MB), extensible, community-driven
- Negative: Users must manually install language grammars (documented in README)

---

## ADR-004: CLI Framework - Commander.js

**Date**: 2025-01-18
**Status**: Accepted
**Context**:
- Three operational modes: `scan`, `pipeline`, `serve`
- Multiple flags per mode (`-l`, `-r`, `-o`, `--ci`, `--only-security`, etc.)
- Need help text generation and argument validation

**Options Considered**:
1. **Manual parsing** - `process.argv` with custom logic
2. **Commander.js** - Mature, widely-used CLI framework
3. **Yargs** - Feature-rich but heavier

**Decision**: Commander.js

**Rationale**:
- Industry standard for Node.js CLIs (used by npm, Vue CLI, etc.)
- Auto-generates `--help` text
- Clean API for subcommands (`scan`, `pipeline`, `serve`)
- Lightweight (< 10 KB)

**Consequences**:
- Positive: Maintainable, well-documented, handles edge cases
- Negative: Adds one dependency (acceptable trade-off)

---

## ADR-005: Report Formats - Console, JSON, HTML

**Date**: 2025-01-20
**Status**: Accepted
**Context**:
- Different users need different outputs:
  - Developers: terminal-friendly console output
  - CI/CD: machine-parsable JSON
  - Team leads: shareable HTML reports

**Options Considered**:
1. **Console only** - Simplest, but not machine-parsable
2. **JSON only** - Machine-friendly, but not human-readable
3. **Multi-format reporter system** - Console + JSON + HTML

**Decision**: Multi-format reporter system

**Rationale**:
- Reporter abstraction (`Reporter` interface) allows pluggable outputs
- Console uses ANSI colors (chalk) for readability
- JSON schema is stable for CI integration
- HTML is self-contained (embedded CSS) for offline viewing

**Consequences**:
- Positive: Flexible, supports all use cases
- Negative: Slightly more code complexity (mitigated by clean abstraction)

---

## ADR-006: Configuration - YAML over JSON

**Date**: 2025-01-22
**Status**: Accepted
**Context**:
- Users need to configure thresholds, exclude patterns, rule toggles
- Config file should be human-editable and version-controllable

**Options Considered**:
1. **JSON** - Standard, but no comments allowed
2. **YAML** - Human-friendly, supports comments
3. **TOML** - Less common in JavaScript ecosystem

**Decision**: YAML (`.codeqcrc.yml`)

**Rationale**:
- YAML allows comments for documenting config choices
- More readable than JSON (no trailing commas, less punctuation)
- Widely used in CI/CD (GitHub Actions, GitLab CI)
- `yaml` npm package is lightweight and fast

**Consequences**:
- Positive: User-friendly, comment support, industry standard
- Negative: YAML parsing is slightly slower than JSON (negligible for config files)

---

## ADR-007: Pipeline Architecture - 10-Step Linear

**Date**: 2025-01-25
**Status**: Accepted
**Context**:
- Code-QC v3.5 §8 defines 10-step pipeline
- Steps must execute sequentially (fail-fast on critical failures)
- Evidence collection required for audit trail

**Options Considered**:
1. **Parallel execution** - Run independent steps concurrently
2. **Linear execution** - One step at a time
3. **DAG (Directed Acyclic Graph)** - Define dependencies

**Decision**: Linear execution with fail-fast

**Rationale**:
- Code-QC §8 explicitly requires linear "走線化" (circuit-like flow)
- Fail-fast aligns with hardware testing principles (stop on first critical fault)
- Evidence files are timestamped sequentially
- Parallel execution would complicate error handling and logging

**Consequences**:
- Positive: Simple, predictable, aligns with spec
- Negative: Slower than parallel (acceptable for release pipeline, not dev loop)

---

## ADR-008: Proof Pack - ZIP + SHA256 + Merkle

**Date**: 2025-01-28
**Status**: Accepted
**Context**:
- LV1-LV9 anti-counterfeiting requires cryptographic proof
- Evidence files must be tamper-evident
- Proof pack should be verifiable offline

**Options Considered**:
1. **Simple ZIP** - No integrity check
2. **ZIP + SHA256** - Single hash for entire archive
3. **ZIP + SHA256 + Merkle root** - Per-file hashes + tree structure

**Decision**: ZIP + SHA256 + Merkle root

**Rationale**:
- Merkle tree allows verification of individual files without unpacking full archive
- SHA256 provides cryptographic assurance (collision-resistant)
- Nonce (UUID v4) prevents replay attacks
- Aligns with LV5 proof standards in Code-QC v3.5

**Consequences**:
- Positive: Audit-grade proof, future-proof for blockchain integration
- Negative: Slightly larger proof files (< 10 KB overhead, negligible)

---

## ADR-009: Self-Detection Avoidance - Path-Based Exclusion

**Date**: 2025-02-01
**Status**: Accepted
**Context**:
- CodeQC test suite contains intentional violations (for testing detectors)
- Scanning CodeQC's own codebase would produce false positives
- Need mechanism to skip test/spec/mock files

**Options Considered**:
1. **No exclusion** - Accept self-flagging as expected
2. **Hardcoded exclusion** - Skip files with `codeqc` in path
3. **Pattern-based exclusion** - Skip files matching `test|spec|mock` regex

**Decision**: Pattern-based exclusion

**Rationale**:
- Generic pattern (`test|spec|mock`) works for all projects, not just CodeQC
- Avoids overfitting to CodeQC's internal structure
- Users can override via config (`excludePatterns`)
- Aligns with industry practice (ESLint, Prettier ignore test files by default)

**Consequences**:
- Positive: Clean scans on own codebase, user-configurable
- Negative: Users must explicitly include test files if desired (rare)

---

## ADR-010: Serve Mode - HTTP + WebSocket

**Date**: 2025-02-05
**Status**: Accepted
**Context**:
- Dashboard needs real-time updates when scans complete
- API must support programmatic scan triggers (e.g., from CI webhook)

**Options Considered**:
1. **HTTP polling** - Dashboard polls `/api/status` every 3 seconds
2. **WebSocket** - Bidirectional, real-time updates
3. **Server-Sent Events (SSE)** - Unidirectional, simpler than WebSocket

**Decision**: HTTP API + WebSocket for dashboard updates

**Rationale**:
- WebSocket provides true real-time updates (< 100 ms latency)
- HTTP API allows stateless scan triggers (`POST /api/scan`)
- WebSocket is widely supported in all modern browsers
- Small overhead (one persistent connection per dashboard client)

**Consequences**:
- Positive: Real-time UX, scalable for 10+ concurrent users
- Negative: More complex than HTTP-only (acceptable for P2 feature)

---

## ADR-011: Node.js Version - Require 18+

**Date**: 2025-02-08
**Status**: Accepted
**Context**:
- CodeQC uses ESM modules, `fetch` API, `crypto` module
- Node.js 18 is LTS until April 2025
- Node.js 20 is current LTS (until April 2026)

**Options Considered**:
1. **Node.js 16+** - Broader compatibility
2. **Node.js 18+** - Stable LTS with modern features
3. **Node.js 20+** - Latest LTS, smaller user base

**Decision**: Node.js 18+

**Rationale**:
- Node.js 18 includes native `fetch` (no polyfill needed)
- ESM support is stable in 18+
- 16 reaches EOL in September 2024 (too soon)
- Most CI environments default to 18 or 20
- `package.json` specifies `"engines": { "node": ">=18.0.0" }`

**Consequences**:
- Positive: Modern APIs, no polyfills, future-proof
- Negative: Users on Node.js 16 must upgrade (documented in README)

---

## ADR-012: Build Tool - tsup over tsc

**Date**: 2025-02-10
**Status**: Accepted
**Context**:
- Need to produce ESM, CJS, and TypeScript declaration files
- Fast incremental builds for development
- Single command for production build

**Options Considered**:
1. **tsc only** - TypeScript compiler, no bundling
2. **Rollup** - Powerful, but complex config
3. **tsup** - Zero-config, esbuild-based, fast

**Decision**: tsup

**Rationale**:
- tsup generates ESM, CJS, and `.d.ts` in one command
- esbuild backend is 10x faster than tsc
- Zero config for standard TypeScript projects
- Widely used in modern npm packages (Vite, Vitest use similar approach)

**Consequences**:
- Positive: Fast builds (< 2s), dual ESM/CJS output, simple
- Negative: Adds build dependency (acceptable, dev-only)

---

## Summary Table

| ADR | Decision | Impact | Reversibility |
|-----|----------|--------|---------------|
| ADR-001 | TypeScript over Python | High | Low (rewrite required) |
| ADR-002 | Regex + AST, No LLM | High | Medium (can add LLM later) |
| ADR-003 | Plugin system | Medium | Medium (can bundle grammars) |
| ADR-004 | Commander.js | Low | High (swap CLI framework) |
| ADR-005 | Multi-format reporters | Medium | High (add/remove formats) |
| ADR-006 | YAML config | Low | High (support JSON too) |
| ADR-007 | Linear pipeline | High | Low (spec-mandated) |
| ADR-008 | ZIP + SHA256 + Merkle | Medium | Low (audit requirement) |
| ADR-009 | Path-based exclusion | Low | High (config-driven) |
| ADR-010 | HTTP + WebSocket | Medium | High (optional mode) |
| ADR-011 | Node.js 18+ | Medium | Medium (can backport) |
| ADR-012 | tsup build tool | Low | High (swap build tool) |

---

*MAIDOS-CodeQC ADR v0.3.5 -- CodeQC Gate C Compliant*
