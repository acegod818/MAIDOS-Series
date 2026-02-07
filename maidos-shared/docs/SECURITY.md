# Security — maidos-shared

## Authentication (maidos-auth)

- HMAC-SHA256 for token signing (hmac + sha2 crates)
- ring for cryptographic primitives
- No plaintext credential storage
- Token expiry enforcement

## Transport Security

- reqwest with rustls-tls (no OpenSSL dependency)
- HTTPS-only for all external API calls
- Certificate validation via webpki-roots

## Blockchain (maidos-chain)

- ethers crate for Ethereum interaction
- Private key management via secure memory (zeroize)
- Transaction signing isolated in dedicated module

## P2P Network (maidos-p2p)

- Encrypted peer connections
- Peer identity verification
- Message integrity via HMAC

## Dependency Auditing

- `cargo audit` run in CI pipeline
- `audit_and_fake_check.rs` integration test validates no known vulnerabilities
- zeroize pinned to 1.8.2 for consistent secret clearing

## Threat Model

| Threat | Mitigation |
|--------|-----------|
| Token forgery | HMAC-SHA256 with server-side secret |
| Man-in-middle | TLS via rustls, cert pinning optional |
| Key leakage | zeroize on drop, no disk persistence |
| Dependency supply chain | cargo audit, lock file review |
