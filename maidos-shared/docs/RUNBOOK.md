# Operational Runbook - maidos-shared

## Overview

This runbook provides step-by-step procedures for diagnosing and resolving common operational issues with maidos-shared libraries.

---

## Issue RB-001: Token Verification Failures

### Symptoms

- `AuthError::InvalidSignature` returned from `verify()`
- Application logs show "Token verification failed"
- Users report authentication rejections

### Root Causes

1. **Secret Key Mismatch**: Issuer and verifier use different secrets
2. **Token Expiration**: Token TTL exceeded
3. **Token Revocation**: Token explicitly revoked
4. **Clock Skew**: System clocks out of sync (>5 minutes)

### Diagnosis

**Step 1: Check Error Type**
```rust
match issuer.verify(&token) {
    Err(AuthError::InvalidSignature) => println!("Secret mismatch"),
    Err(AuthError::TokenExpired) => println!("Token expired"),
    Err(AuthError::TokenRevoked) => println!("Token revoked"),
    Ok(_) => println!("Valid token"),
}
```

**Step 2: Verify Secret Key**
```bash
# Check environment variable
echo $MAIDOS_AUTH_SECRET

# Verify length (must be >= 32 bytes)
echo -n "$MAIDOS_AUTH_SECRET" | wc -c
```

**Step 3: Inspect Token Claims**
```bash
# Decode JWT (without verification)
echo "$TOKEN" | cut -d'.' -f2 | base64 -d | jq
# Check "exp" (expiration timestamp)
```

**Step 4: Check System Clock**
```bash
# Compare issuer and verifier clocks
date -u +%s
# Timestamps should differ by < 5 minutes
```

### Resolution

**Scenario 1: Secret Key Mismatch**
```bash
# Ensure both issuer and verifier use same secret
export MAIDOS_AUTH_SECRET="same-secret-on-all-nodes"

# Restart services
systemctl restart maidos-driver
systemctl restart maidos-forge
```

**Scenario 2: Token Expiration**
```rust
// Issue new token with longer TTL
let token = issuer.issue(user_id, caps, Duration::from_hours(24))?;
```

**Scenario 3: Token Revocation**
```rust
// Check revocation list
if issuer.is_revoked(&token)? {
    // Issue new token
    let new_token = issuer.issue(user_id, caps, ttl)?;
}
```

**Scenario 4: Clock Skew**
```bash
# Sync system clock (Linux)
sudo ntpdate -u pool.ntp.org

# Sync system clock (Windows)
w32tm /resync
```

### Verification

```bash
# Test token verification
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/verify
# Expected: 200 OK
```

---

## Issue RB-002: LLM Provider Connection Failures

### Symptoms

- `LlmError::ConnectionFailed` or `LlmError::Timeout`
- Application logs show "Failed to connect to provider"
- LLM requests hang indefinitely

### Root Causes

1. **Invalid API Key**: API key expired or incorrect
2. **Network Timeout**: Provider unreachable (firewall, DNS)
3. **Rate Limit Exceeded**: Too many requests in short time
4. **Provider Outage**: Cloud provider service disruption

### Diagnosis

**Step 1: Test Provider Connectivity**
```bash
# OpenAI
curl -H "Authorization: Bearer $OPENAI_API_KEY" \
     https://api.openai.com/v1/models

# Anthropic
curl -H "x-api-key: $ANTHROPIC_API_KEY" \
     https://api.anthropic.com/v1/messages \
     -d '{"model":"claude-3-5-sonnet-20241022","messages":[{"role":"user","content":"test"}],"max_tokens":10}'

# Ollama (local)
curl http://localhost:11434/api/version
```

**Step 2: Check API Key**
```bash
# Verify API key is set
echo $OPENAI_API_KEY | grep -o "^sk-[A-Za-z0-9]*"

# Check key expiration (if provider supports it)
curl -H "Authorization: Bearer $OPENAI_API_KEY" \
     https://api.openai.com/v1/usage
```

**Step 3: Check Rate Limits**
```bash
# OpenAI rate limit headers
curl -i -H "Authorization: Bearer $OPENAI_API_KEY" \
     https://api.openai.com/v1/models \
     | grep -i "x-ratelimit"
```

**Step 4: Test with Fallback Provider**
```rust
use maidos_llm::{Router, RoutingStrategy};

// Add fallback provider
let router = Router::new(vec![openai, claude, ollama])
    .with_strategy(RoutingStrategy::Fallback);

let response = router.route(request).await?;
// If succeeds, primary provider has issue
```

### Resolution

**Scenario 1: Invalid API Key**
```bash
# Rotate API key
# 1. Generate new key from provider dashboard
# 2. Update environment variable
export OPENAI_API_KEY="sk-new-key-here"

# 3. Restart application
systemctl restart myapp
```

**Scenario 2: Network Timeout**
```bash
# Check DNS resolution
nslookup api.openai.com

# Check firewall rules
sudo iptables -L -n | grep 443

# Test with longer timeout
curl --max-time 60 https://api.openai.com/v1/models
```

**Scenario 3: Rate Limit Exceeded**
```rust
// Implement exponential backoff
use tokio::time::{sleep, Duration};

let mut retries = 0;
loop {
    match provider.complete(request.clone()).await {
        Ok(response) => break response,
        Err(LlmError::RateLimitExceeded) if retries < 5 => {
            let delay = 2u64.pow(retries) * 1000;  // 1s, 2s, 4s, 8s, 16s
            sleep(Duration::from_millis(delay)).await;
            retries += 1;
        }
        Err(e) => return Err(e),
    }
}
```

**Scenario 4: Provider Outage**
```bash
# Check provider status page
# OpenAI: https://status.openai.com
# Anthropic: https://status.anthropic.com

# Use fallback provider
# (automatic if using Router with Fallback strategy)
```

### Verification

```bash
# Test LLM request
curl -X POST http://localhost:8080/api/llm/complete \
     -H "Content-Type: application/json" \
     -d '{"prompt":"Hello","model":"gpt-4o"}'
# Expected: {"text":"Hello! How can I...","usage":{...}}
```

---

## Issue RB-003: Event Bus Disconnections

### Symptoms

- `BusError::Disconnected` returned from `receive()`
- Publisher/subscriber logs show "Connection lost"
- Messages not delivered between services

### Root Causes

1. **Publisher Restart**: Publisher service restarted or crashed
2. **Network Partition**: Network between publisher and subscriber interrupted
3. **Port Conflict**: Another process bound to ZeroMQ port
4. **Resource Exhaustion**: Too many open connections

### Diagnosis

**Step 1: Check Publisher Status**
```bash
# Check if publisher process is running
ps aux | grep publisher

# Check publisher logs
journalctl -u maidos-publisher -n 50
```

**Step 2: Test ZeroMQ Port**
```bash
# Check if port is listening
netstat -tuln | grep 5555

# Test connection
telnet localhost 5555
```

**Step 3: Check Subscriber Reconnection**
```rust
// Enable debug logging
use maidos_log::init_logger;
init_logger(LogLevel::Debug)?;

// Subscriber should log reconnection attempts
// "Attempting reconnect (1/5)..."
```

**Step 4: Monitor Connection Count**
```bash
# Check open connections
lsof -i :5555 | wc -l

# Check file descriptor limit
ulimit -n
```

### Resolution

**Scenario 1: Publisher Restart**
```bash
# Restart publisher
systemctl restart maidos-publisher

# Verify subscribers reconnect automatically
# (should happen within 30 seconds)
journalctl -u maidos-subscriber -f
# Look for "Reconnected successfully"
```

**Scenario 2: Network Partition**
```bash
# Check network connectivity
ping <publisher_host>

# Check routing
traceroute <publisher_host>

# Check firewall
sudo iptables -L -n | grep 5555
```

**Scenario 3: Port Conflict**
```bash
# Find process using port
sudo lsof -i :5555

# Kill conflicting process
sudo kill -9 <PID>

# Restart publisher on correct port
```

**Scenario 4: Resource Exhaustion**
```bash
# Increase file descriptor limit
echo "* soft nofile 65536" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65536" | sudo tee -a /etc/security/limits.conf

# Restart system or re-login
ulimit -n  # Should show 65536
```

### Verification

```bash
# Test message publish/subscribe
# Terminal 1: Start subscriber
maidos-bus-subscriber tcp://localhost:5555 "test."

# Terminal 2: Publish message
maidos-bus-publisher tcp://*:5555 "test.msg" '{"data":"hello"}'

# Terminal 1 should receive: {"data":"hello"}
```

---

## Issue RB-004: Config Hot Reload Not Triggering

### Symptoms

- Config file updated but application uses old values
- No "Config reloaded" log message after file change
- Application requires manual restart to pick up changes

### Root Causes

1. **File Watcher Not Started**: Hot reload not enabled
2. **File System Events Dropped**: Too many rapid edits
3. **Validation Failure**: New config invalid, old config retained
4. **Permission Issue**: Application cannot read updated file

### Diagnosis

**Step 1: Check Hot Reload Status**
```rust
// Verify hot reload is enabled
let loader = ConfigLoader::new()
    .with_hot_reload(true)?;  // Must be true
```

**Step 2: Monitor File Watcher**
```bash
# Check file system events (Linux)
sudo apt-get install inotify-tools
inotifywait -m config.toml

# Make change
echo "# test" >> config.toml
# Should see "MODIFY config.toml"
```

**Step 3: Check Application Logs**
```bash
# Look for reload attempts
journalctl -u myapp | grep -i "reload"

# Look for validation errors
journalctl -u myapp | grep -i "config.*error"
```

**Step 4: Test Manual Reload**
```rust
// Force reload
loader.reload()?;
// If this works, file watcher is the issue
```

### Resolution

**Scenario 1: File Watcher Not Started**
```rust
// Enable hot reload
let loader = ConfigLoader::new()
    .with_hot_reload(true)?;

// Verify watcher is running
assert!(loader.is_watching());
```

**Scenario 2: File System Events Dropped**
```bash
# Increase inotify limits (Linux)
echo "fs.inotify.max_user_watches=524288" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p

# Use debounced editor saves (vim)
# Add to ~/.vimrc:
set noswapfile
set updatetime=500
```

**Scenario 3: Validation Failure**
```bash
# Test config file manually
maidos-config-validate config.toml

# Fix validation errors
# Common issues: missing required fields, type mismatches

# Verify with schema
maidos-config-validate --schema schema.toml config.toml
```

**Scenario 4: Permission Issue**
```bash
# Check file permissions
ls -la config.toml

# Fix permissions
chmod 644 config.toml
chown myapp:myapp config.toml
```

### Verification

```bash
# Edit config file
echo "log_level = \"debug\"" >> config.toml

# Wait 1 second (debounce period)
sleep 1

# Check if application picked up change
curl http://localhost:8080/config | jq '.log_level'
# Expected: "debug"
```

---

## Issue RB-005: FFI Memory Leaks in C# Application

### Symptoms

- Application memory usage grows over time
- No memory returned after disposing objects
- Windows Task Manager shows increasing private bytes

### Root Causes

1. **Missing `_free` Calls**: Caller forgot to free returned strings/pointers
2. **Exception Before Free**: C# exception thrown before `_free` call
3. **Circular References**: Provider/subscriber not disposed

### Diagnosis

**Step 1: Use Memory Profiler**
```csharp
// Use Visual Studio Diagnostic Tools
// Debug > Performance Profiler > .NET Object Allocation Tracking
// Run application and look for growing allocations
```

**Step 2: Check for Missing `_free` Calls**
```csharp
// Search code for DllImport calls without corresponding _free
// BAD:
IntPtr token = maidos_auth_issue_token(...);
string tokenStr = Marshal.PtrToStringUTF8(token);
// Missing: maidos_auth_free_string(token);

// GOOD:
IntPtr token = maidos_auth_issue_token(...);
try {
    string tokenStr = Marshal.PtrToStringUTF8(token);
    // Use tokenStr
} finally {
    maidos_auth_free_string(token);
}
```

**Step 3: Test with Leak Detector**
```bash
# Windows: Use Application Verifier
# Linux: Use Valgrind
valgrind --leak-check=full --show-leak-kinds=all ./myapp
```

### Resolution

**Scenario 1: Missing `_free` Calls**
```csharp
// Wrap FFI calls in using statement
public class MaidosToken : IDisposable {
    private IntPtr _ptr;

    public MaidosToken(string userId, string[] caps, string secret, int ttl) {
        _ptr = MaidosAuth.maidos_auth_issue_token(userId, caps, caps.Length, secret, ttl);
        if (_ptr == IntPtr.Zero)
            throw new InvalidOperationException(GetLastError());
    }

    public string GetToken() {
        return Marshal.PtrToStringUTF8(_ptr);
    }

    public void Dispose() {
        if (_ptr != IntPtr.Zero) {
            MaidosAuth.maidos_auth_free_string(_ptr);
            _ptr = IntPtr.Zero;
        }
    }
}

// Usage
using (var token = new MaidosToken("user", caps, "secret", 3600)) {
    string tokenStr = token.GetToken();
    // Use tokenStr
}  // Automatically freed
```

**Scenario 2: Exception Before Free**
```csharp
// Use try-finally
IntPtr response = IntPtr.Zero;
try {
    response = maidos_llm_complete(llm, "gpt-4o", "Hello");
    string text = Marshal.PtrToStringUTF8(response);
    ProcessResponse(text);  // May throw
} finally {
    if (response != IntPtr.Zero)
        maidos_llm_free_string(response);
}
```

**Scenario 3: Circular References**
```csharp
// Implement IDisposable for wrappers
public class MaidosLlmProvider : IDisposable {
    private IntPtr _provider;

    ~MaidosLlmProvider() {
        Dispose();
    }

    public void Dispose() {
        if (_provider != IntPtr.Zero) {
            maidos_llm_free_provider(_provider);
            _provider = IntPtr.Zero;
        }
        GC.SuppressFinalize(this);
    }
}
```

### Verification

```csharp
// Memory test
var initialMemory = GC.GetTotalMemory(true);

for (int i = 0; i < 10000; i++) {
    using (var token = new MaidosToken(...)) {
        _ = token.GetToken();
    }
}

GC.Collect();
GC.WaitForPendingFinalizers();
var finalMemory = GC.GetTotalMemory(true);

// Memory should not grow significantly
Console.WriteLine($"Memory delta: {(finalMemory - initialMemory) / 1024.0} KB");
// Expected: < 100 KB
```

---

## Escalation Procedures

### Level 1: Application Team
- Initial diagnosis using this runbook
- Check logs, metrics, configuration
- Attempt documented resolutions

### Level 2: Platform Team
- Deep dive into maidos-shared internals
- Review source code, debug symbols
- Reproduce issue in isolated environment

### Level 3: MAIDOS Core Team
- File GitHub issue with:
  - Minimal reproduction case
  - Debug logs (`RUST_LOG=debug`)
  - Environment details (OS, Rust version, crate versions)
  - Expected vs actual behavior

**GitHub Issue Template**:
```markdown
### Description
[Clear description of issue]

### Steps to Reproduce
1. [Step 1]
2. [Step 2]

### Expected Behavior
[What should happen]

### Actual Behavior
[What actually happens]

### Environment
- OS: [e.g., Windows 11, Ubuntu 22.04]
- Rust: [e.g., 1.75.0]
- maidos-shared: [e.g., 0.2.0]
- Crate: [e.g., maidos-auth, maidos-llm]

### Logs
```
[Paste relevant logs]
```
```

---

## Emergency Contacts

- **On-Call Engineer**: Slack #maidos-oncall
- **GitHub Issues**: https://github.com/maidos/maidos-shared/issues
- **Security Issues**: security@maidos.dev (PGP key: https://maidos.dev/pgp.txt)
