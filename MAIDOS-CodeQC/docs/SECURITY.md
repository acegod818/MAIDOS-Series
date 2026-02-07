# MAIDOS-CodeQC -- Security Policy

| Field     | Value              |
|-----------|--------------------|
| Product   | MAIDOS-CodeQC      |
| Version   | v3.0               |
| Type      | Security Policy    |

## Security Model

MAIDOS-CodeQC operates as a local development tool that inspects and validates project artifacts. It does not handle user credentials or sensitive data directly, but it does execute build and test commands.

## Threat Assessment

| Threat                      | Risk   | Mitigation                                     |
|-----------------------------|--------|-------------------------------------------------|
| Malicious build commands    | Medium | Commands sourced only from project config       |
| Path traversal              | Low    | All paths validated and sandboxed               |
| Dependency supply chain     | Medium | npm audit, lockfile integrity checks            |
| Evidence tampering          | Low    | SHA-256 hashes in manifest                      |
| Web UI unauthorized access  | Low    | Dashboard binds to localhost only               |

## Evidence Integrity

| Measure                   | Description                                    |
|---------------------------|------------------------------------------------|
| SHA-256 hashing           | All evidence files are hashed in the manifest  |
| Timestamp recording       | ISO 8601 timestamps for all gate executions    |
| Immutable proof packs     | Generated packs are read-only after creation   |

## Web UI Security

| Measure                    | Description                                   |
|----------------------------|-----------------------------------------------|
| Localhost binding          | Dashboard only accessible on 127.0.0.1        |
| No authentication          | Local-only access, no credentials needed      |
| No external requests       | Dashboard does not make external API calls     |

## Dependency Management

- `npm audit` is run as part of the CI pipeline
- `package-lock.json` is committed and verified
- No `postinstall` scripts from untrusted packages

## Reporting Vulnerabilities

Report security issues to: security@maidos.dev

*MAIDOS-CodeQC SECURITY v3.0 -- CodeQC Gate C Compliant*
