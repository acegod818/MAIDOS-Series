# MAIDOS-CodeQC -- Architecture Decision Records

> **Version**: 1.0
> **Date**: 2026-02-07
> **Product**: MAIDOS-CodeQC v2.6.1

---

## ADR-001: Rust as the Implementation Language

**Status**: Accepted
**Date**: 2025-10-15

**Context**: MAIDOS-CodeQC needs to scan large codebases quickly while maintaining
memory safety. The tool must produce deterministic results and run on Windows, Linux,
and macOS without a managed runtime.

**Decision**: Implement the core engine and all first-party plugins in Rust.

**Rationale**:
- Memory safety without garbage collection eliminates a class of runtime bugs.
- Zero-cost abstractions enable performance on par with C/C++.
- The `libloading` crate provides mature cross-platform dynamic library support.
- Rust's type system enforces correctness in the rule engine and plugin API.
- Single static binary distribution simplifies deployment.

**Consequences**:
- Plugin authors must use Rust or provide a C-ABI-compatible shared library.
- Build times are longer than scripting alternatives; mitigated by incremental compilation.

---

## ADR-002: Plugin Architecture for Language Support

**Status**: Accepted
**Date**: 2025-10-20

**Context**: MAIDOS-Series products span Rust, C#, TypeScript, Java, Kotlin, Swift,
Python, Go, and configuration languages. A monolithic analyzer cannot scale to all
ecosystems without becoming unwieldy.

**Decision**: Use a plugin architecture where each language ecosystem is a separate
shared library implementing a common trait interface.

**Rationale**:
- Separation of concerns: each plugin owns its parser and rule set.
- Independent release cycles: a plugin can be updated without rebuilding the core.
- Hot-loading: plugins can be added at runtime for extensibility.
- Reduces binary size: users load only the plugins they need.

**Consequences**:
- FFI boundary requires careful ABI stability management.
- Plugin API must be versioned; breaking changes require a major version bump.
- Testing must cover both isolated plugin tests and integration with the core.

**Plugins (v2.6.1)**:
`config`, `data`, `dotnet`, `enterprise`, `functional`, `jvm`, `mobile`,
`scripting`, `systems`, `web`

---

## ADR-003: Progressive Gate System (G1-G4)

**Status**: Accepted
**Date**: 2025-11-01

**Context**: Enforcing all quality rules at once creates an overwhelming wall of
violations for legacy codebases. Teams need a gradual adoption path.

**Decision**: Implement a 4-gate progressive quality system where each gate builds
on the previous one.

**Rationale**:
- G1 (Hygiene) catches the most critical issues: fake implementations, stub code.
- G2 (Correctness) adds type safety and error handling checks.
- G3 (Maintainability) enforces complexity and documentation standards.
- G4 (Release) is the full bar required for production deployment.
- Teams can start at G1 and incrementally raise the bar.

**Consequences**:
- Each rule must be tagged with its gate level.
- Configuration allows setting the minimum gate for pass/fail evaluation.
- CI pipelines can enforce different gates for different branches (e.g., G2 for
  feature branches, G4 for main).

---

*New ADRs are appended to this file with sequential numbering.*
