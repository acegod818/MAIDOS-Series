# maidos-shared — Operational Runbook

## 1. Purpose

This runbook provides step-by-step procedures for common operational tasks and
troubleshooting scenarios related to maidos-shared.

---

## 2. Dependency Management

### 2.1 Adding a New Dependency

1. Add the dependency to the workspace-level `[workspace.dependencies]` in root `Cargo.toml`.
2. Reference it in the sub-crate's `Cargo.toml` with `dep.workspace = true`.
3. Verify license compatibility (MIT/Apache-2.0).
4. Run `cargo audit` to check for known vulnerabilities.
5. Run `cargo test --workspace` to confirm no regressions.

### 2.2 Updating Dependencies

```bash
cargo update                       # Update all within semver constraints
cargo audit                        # Check for vulnerabilities
cargo test --workspace             # Verify nothing breaks
```

### 2.3 Removing a Dependency

1. Remove from workspace-level `Cargo.toml`.
2. Remove from sub-crate `Cargo.toml`.
3. Run `cargo build --workspace` to find and fix all broken imports.

---

## 3. Troubleshooting

### 3.1 "feature `X` not found" Error

**Cause:** Consumer enabled a feature that does not exist in the current version.
**Fix:** Check available features in `maidos-shared/Cargo.toml`. Ensure the consumer's
`Cargo.lock` points to the correct version.

### 3.2 Auth Token Refresh Fails

**Symptoms:** `AuthError::TokenExpired` despite `ensure_valid_token()` calls.
**Steps:**
1. Check network connectivity to the OAuth2 provider.
2. Verify `redirect_uri` matches the registered application.
3. Inspect logs: `RUST_LOG=maidos_auth=debug cargo run`.
4. Clear cached tokens: delete `~/.maidos/tokens.json` and re-authenticate.

### 3.3 Ollama Connection Refused

**Symptoms:** `LlmError::ConnectionRefused` on `generate()`.
**Steps:**
1. Confirm Ollama is running: `curl http://localhost:11434/api/tags`.
2. Check endpoint in config matches Ollama's bind address.
3. Verify firewall rules allow localhost connections.
4. Restart Ollama: `ollama serve`.

### 3.4 Message Bus Messages Not Delivered

**Symptoms:** Subscriber handler is never invoked.
**Steps:**
1. Confirm the topic string matches exactly (case-sensitive).
2. Verify `subscribe()` was called before `publish()`.
3. Check bus state: `bus.state()` should return `Running`.
4. Enable debug logging: `RUST_LOG=maidos_bus=trace`.

### 3.5 P2P Peer Discovery Fails

**Symptoms:** No peers found after extended wait.
**Steps:**
1. Ensure both nodes are on the same network subnet.
2. Verify mDNS is not blocked by firewall (UDP port 5353).
3. Check `PeerConfig` has matching service name.
4. Try manual bootstrap: add known peer address to config.

### 3.6 Build Time Regression

**Symptoms:** `cargo build --workspace` takes significantly longer.
**Steps:**
1. Run `cargo build --timings` to identify slow crates.
2. Check for newly added proc-macro dependencies.
3. Ensure `sccache` or equivalent is configured.
4. Consider splitting large sub-crates into smaller units.

---

## 4. Emergency Contacts

| Role | Responsibility |
|------|---------------|
| Workspace maintainer | Version bumps, publish coordination |
| Sub-crate owner | Bug fixes within their sub-crate |
| CI administrator | Pipeline failures, runner provisioning |
