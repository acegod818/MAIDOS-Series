# FFI Contracts - maidos-shared

## Overview

maidos-shared exposes C-compatible FFI functions for cross-language interop. All FFI functions follow these conventions:

- **Calling Convention**: `cdecl` (C default)
- **String Encoding**: UTF-8 (not null-terminated, length-prefixed internally)
- **Memory Ownership**: Caller frees returned pointers via `_free` functions
- **Error Handling**: Return `NULL` or negative codes on error
- **Thread Safety**: All functions are thread-safe unless noted

## maidos-auth FFI

### maidos_auth_issue_token

```c
char* maidos_auth_issue_token(
    const char* user_id,
    const char* const* capabilities,
    int capabilities_len,
    const char* secret,
    int ttl_seconds
);
```

**Description**: Issue a new authentication token.

**Parameters**:
- `user_id`: User identifier (UTF-8 string)
- `capabilities`: Array of capability names (e.g., `["read_user", "write_driver"]`)
- `capabilities_len`: Number of capabilities
- `secret`: HMAC-SHA256 secret key (>= 32 bytes)
- `ttl_seconds`: Time-to-live in seconds (max 86400 = 24 hours)

**Returns**: JWT token string (caller must free via `maidos_auth_free_string`), or `NULL` on error.

**Example (C)**:
```c
const char* caps[] = {"read_user", "write_driver"};
char* token = maidos_auth_issue_token("user_123", caps, 2, "my-secret", 3600);
if (token == NULL) {
    fprintf(stderr, "Failed to issue token\n");
    return -1;
}
printf("Token: %s\n", token);
maidos_auth_free_string(token);
```

**Example (C#)**:
```csharp
[DllImport("maidos_shared.dll", CallingConvention = CallingConvention.Cdecl)]
public static extern IntPtr maidos_auth_issue_token(
    [MarshalAs(UnmanagedType.LPUTF8Str)] string userId,
    [MarshalAs(UnmanagedType.LPArray, ArraySubType = UnmanagedType.LPUTF8Str)] string[] capabilities,
    int capabilitiesLen,
    [MarshalAs(UnmanagedType.LPUTF8Str)] string secret,
    int ttlSeconds
);

string[] caps = { "read_user", "write_driver" };
IntPtr tokenPtr = maidos_auth_issue_token("user_123", caps, caps.Length, "my-secret", 3600);
string token = Marshal.PtrToStringUTF8(tokenPtr);
maidos_auth_free_string(tokenPtr);
```

---

### maidos_auth_verify_token

```c
char* maidos_auth_verify_token(
    const char* token,
    const char* secret
);
```

**Description**: Verify a token's signature and expiration.

**Parameters**:
- `token`: JWT token string
- `secret`: HMAC-SHA256 secret key

**Returns**: User ID (UTF-8 string, caller must free), or `NULL` if invalid/expired.

**Example (C)**:
```c
char* user_id = maidos_auth_verify_token(token, "my-secret");
if (user_id == NULL) {
    fprintf(stderr, "Token verification failed\n");
    return -1;
}
printf("Authenticated: %s\n", user_id);
maidos_auth_free_string(user_id);
```

---

### maidos_auth_has_capability

```c
int maidos_auth_has_capability(
    const char* token,
    const char* secret,
    const char* capability
);
```

**Description**: Check if a token has a specific capability.

**Parameters**:
- `token`: JWT token string
- `secret`: HMAC-SHA256 secret key
- `capability`: Capability name (e.g., "write_driver")

**Returns**: 1 if token has capability, 0 if not, -1 on error.

**Example (C)**:
```c
int has_write = maidos_auth_has_capability(token, "my-secret", "write_driver");
if (has_write == 1) {
    install_driver();
} else {
    fprintf(stderr, "Insufficient permissions\n");
}
```

---

### maidos_auth_free_string

```c
void maidos_auth_free_string(char* s);
```

**Description**: Free a string returned by auth functions.

**Parameters**:
- `s`: String pointer to free

---

## maidos-llm FFI

### maidos_llm_create_provider

```c
void* maidos_llm_create_provider(
    const char* provider_type,
    const char* api_key,
    const char* base_url
);
```

**Description**: Create an LLM provider instance.

**Parameters**:
- `provider_type`: Provider name ("openai", "anthropic", "google", "ollama", etc.)
- `api_key`: API key (NULL for local providers)
- `base_url`: Custom base URL (NULL for default)

**Returns**: Opaque provider pointer, or `NULL` on error.

**Example (C)**:
```c
void* llm = maidos_llm_create_provider("openai", "sk-...", NULL);
if (llm == NULL) {
    fprintf(stderr, "Failed to create provider\n");
    return -1;
}
```

---

### maidos_llm_complete

```c
char* maidos_llm_complete(
    void* provider,
    const char* model,
    const char* prompt
);
```

**Description**: Send a completion request to the LLM provider.

**Parameters**:
- `provider`: Provider pointer from `maidos_llm_create_provider`
- `model`: Model name (e.g., "gpt-4o", "claude-3-5-sonnet-20241022")
- `prompt`: User prompt (UTF-8 string)

**Returns**: Response text (caller must free via `maidos_llm_free_string`), or `NULL` on error.

**Example (C)**:
```c
char* response = maidos_llm_complete(llm, "gpt-4o", "Why is the sky blue?");
if (response == NULL) {
    fprintf(stderr, "LLM request failed\n");
    return -1;
}
printf("Response: %s\n", response);
maidos_llm_free_string(response);
```

**Example (C#)**:
```csharp
[DllImport("maidos_shared.dll", CallingConvention = CallingConvention.Cdecl)]
public static extern IntPtr maidos_llm_complete(
    IntPtr provider,
    [MarshalAs(UnmanagedType.LPUTF8Str)] string model,
    [MarshalAs(UnmanagedType.LPUTF8Str)] string prompt
);

IntPtr responsePtr = maidos_llm_complete(llm, "gpt-4o", "Why is the sky blue?");
string response = Marshal.PtrToStringUTF8(responsePtr);
Console.WriteLine(response);
maidos_llm_free_string(responsePtr);
```

---

### maidos_llm_free_provider

```c
void maidos_llm_free_provider(void* provider);
```

**Description**: Free a provider instance.

**Parameters**:
- `provider`: Provider pointer

---

### maidos_llm_free_string

```c
void maidos_llm_free_string(char* s);
```

**Description**: Free a string returned by LLM functions.

**Parameters**:
- `s`: String pointer to free

---

## maidos-bus FFI

### maidos_bus_create_publisher

```c
void* maidos_bus_create_publisher(const char* bind_address);
```

**Description**: Create an event bus publisher.

**Parameters**:
- `bind_address`: ZeroMQ bind address (e.g., "tcp://*:5555")

**Returns**: Opaque publisher pointer, or `NULL` on error.

**Example (C)**:
```c
void* publisher = maidos_bus_create_publisher("tcp://*:5555");
```

---

### maidos_bus_publish

```c
int maidos_bus_publish(
    void* publisher,
    const char* topic,
    const char* payload_json
);
```

**Description**: Publish a message to the bus.

**Parameters**:
- `publisher`: Publisher pointer
- `topic`: Topic string (e.g., "job.created")
- `payload_json`: Message payload as JSON string

**Returns**: 0 on success, -1 on error.

**Example (C)**:
```c
int result = maidos_bus_publish(publisher, "job.created", "{\"job_id\":\"j_001\"}");
if (result != 0) {
    fprintf(stderr, "Failed to publish\n");
}
```

---

### maidos_bus_create_subscriber

```c
void* maidos_bus_create_subscriber(const char* connect_address, const char* topic);
```

**Description**: Create an event bus subscriber.

**Parameters**:
- `connect_address`: ZeroMQ connect address (e.g., "tcp://localhost:5555")
- `topic`: Topic prefix to subscribe (e.g., "job.")

**Returns**: Opaque subscriber pointer, or `NULL` on error.

---

### maidos_bus_receive

```c
char* maidos_bus_receive(void* subscriber);
```

**Description**: Receive the next message (blocking).

**Parameters**:
- `subscriber`: Subscriber pointer

**Returns**: JSON message payload (caller must free via `maidos_bus_free_string`), or `NULL` on error.

**Example (C)**:
```c
while (1) {
    char* msg = maidos_bus_receive(subscriber);
    if (msg == NULL) break;
    printf("Received: %s\n", msg);
    maidos_bus_free_string(msg);
}
```

---

### maidos_bus_free_publisher

```c
void maidos_bus_free_publisher(void* publisher);
```

---

### maidos_bus_free_subscriber

```c
void maidos_bus_free_subscriber(void* subscriber);
```

---

### maidos_bus_free_string

```c
void maidos_bus_free_string(char* s);
```

---

## maidos-config FFI

### maidos_config_load

```c
void* maidos_config_load(const char* path);
```

**Description**: Load a TOML config file.

**Parameters**:
- `path`: File path to TOML config

**Returns**: Opaque config pointer, or `NULL` on error.

**Example (C)**:
```c
void* config = maidos_config_load("app.toml");
if (config == NULL) {
    fprintf(stderr, "Failed to load config\n");
    return -1;
}
```

---

### maidos_config_get_string

```c
char* maidos_config_get_string(void* config, const char* key);
```

**Description**: Get a string value from config.

**Parameters**:
- `config`: Config pointer
- `key`: Config key (e.g., "database.url")

**Returns**: String value (caller must free via `maidos_config_free_string`), or `NULL` if key not found.

**Example (C)**:
```c
char* db_url = maidos_config_get_string(config, "database.url");
printf("DB URL: %s\n", db_url);
maidos_config_free_string(db_url);
```

---

### maidos_config_get_int

```c
int maidos_config_get_int(void* config, const char* key, int default_value);
```

**Description**: Get an integer value from config.

**Parameters**:
- `config`: Config pointer
- `key`: Config key
- `default_value`: Default if key not found

**Returns**: Integer value.

---

### maidos_config_free

```c
void maidos_config_free(void* config);
```

---

### maidos_config_free_string

```c
void maidos_config_free_string(char* s);
```

---

## Error Handling

All FFI functions return `NULL` or negative codes on error. To get detailed error messages, call:

```c
const char* maidos_get_last_error();
```

**Description**: Get the last error message from any maidos FFI function.

**Returns**: Static string (do not free), or empty string if no error.

**Example (C)**:
```c
void* llm = maidos_llm_create_provider("invalid", NULL, NULL);
if (llm == NULL) {
    fprintf(stderr, "Error: %s\n", maidos_get_last_error());
}
```

---

## Thread Safety

All FFI functions are thread-safe. However, individual provider/publisher/subscriber instances are **not** thread-safe and must be used from a single thread or with external synchronization.

**Example (C#)**:
```csharp
// Thread-safe: Multiple threads can create separate providers
Task.Run(() => {
    var llm1 = maidos_llm_create_provider("openai", "sk-...", null);
    // Use llm1
});
Task.Run(() => {
    var llm2 = maidos_llm_create_provider("anthropic", "sk-ant-...", null);
    // Use llm2
});

// NOT thread-safe: Multiple threads sharing one provider
var shared_llm = maidos_llm_create_provider("openai", "sk-...", null);
Task.Run(() => maidos_llm_complete(shared_llm, "gpt-4o", "Hello"));  // RACE CONDITION
Task.Run(() => maidos_llm_complete(shared_llm, "gpt-4o", "World"));  // RACE CONDITION
```

---

## Memory Management Rules

1. **Rust allocates, caller frees**: All returned strings/pointers must be freed via corresponding `_free` function.
2. **Input strings**: Caller owns, Rust copies internally.
3. **Null pointers**: Always check for `NULL` return values.
4. **Opaque pointers**: Do not dereference or cast. Use only with provided functions.

**Example (C#)**:
```csharp
// CORRECT: Free after use
IntPtr tokenPtr = maidos_auth_issue_token(...);
string token = Marshal.PtrToStringUTF8(tokenPtr);
maidos_auth_free_string(tokenPtr);  // Must free

// INCORRECT: Memory leak
IntPtr tokenPtr = maidos_auth_issue_token(...);
string token = Marshal.PtrToStringUTF8(tokenPtr);
// Missing maidos_auth_free_string(tokenPtr) - LEAK
```
