/**
 * MAIDOS Shared Core - C API Header
 * 
 * Version: 0.1.0
 * License: MIT
 * 
 * This header provides C bindings for the MAIDOS Shared Core library.
 * Link with: -lmaidos_shared
 */

#ifndef MAIDOS_H
#define MAIDOS_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================================
 * Common Types
 * ============================================================================ */

/** Result code for operations */
typedef enum {
    MAIDOS_OK = 0,
    MAIDOS_ERR_NULL_POINTER = 1,
    MAIDOS_ERR_INVALID_UTF8 = 2,
    MAIDOS_ERR_NOT_FOUND = 3,
    MAIDOS_ERR_INVALID_FORMAT = 4,
    MAIDOS_ERR_IO = 5,
    MAIDOS_ERR_AUTH = 6,
    MAIDOS_ERR_NETWORK = 7,
    MAIDOS_ERR_PROVIDER = 8,
    MAIDOS_ERR_BUDGET = 9,
    MAIDOS_ERR_UNKNOWN = 255
} MaidosResult;

/* ============================================================================
 * maidos-config
 * ============================================================================ */

/** Opaque configuration handle */
typedef struct MaidosConfig MaidosConfig;

/**
 * Load configuration from a TOML file.
 * 
 * @param path Path to the TOML configuration file
 * @return Configuration handle, or NULL on error
 * 
 * @note Caller must free with maidos_config_free()
 */
MaidosConfig* maidos_config_load(const char* path);

/**
 * Parse configuration from a TOML string.
 * 
 * @param toml TOML configuration string
 * @return Configuration handle, or NULL on error
 */
MaidosConfig* maidos_config_from_str(const char* toml);

/**
 * Get a string value from configuration.
 * 
 * @param config Configuration handle
 * @param key Dot-separated key (e.g., "llm.default_provider")
 * @return Value string, or NULL if not found
 * 
 * @note Returned string must be freed with maidos_string_free()
 */
const char* maidos_config_get_string(const MaidosConfig* config, const char* key);

/**
 * Get an integer value from configuration.
 * 
 * @param config Configuration handle
 * @param key Dot-separated key
 * @return Value, or 0 if not found
 */
int64_t maidos_config_get_int(const MaidosConfig* config, const char* key);

/**
 * Get a float value from configuration.
 * 
 * @param config Configuration handle
 * @param key Dot-separated key
 * @return Value, or 0.0 if not found
 */
double maidos_config_get_float(const MaidosConfig* config, const char* key);

/**
 * Export configuration as JSON.
 * 
 * @param config Configuration handle
 * @return JSON string, or NULL on error
 * 
 * @note Returned string must be freed with maidos_string_free()
 */
const char* maidos_config_to_json(const MaidosConfig* config);

/**
 * Reload configuration from file.
 * 
 * @param config Configuration handle
 * @return MAIDOS_OK on success
 */
MaidosResult maidos_config_reload(MaidosConfig* config);

/**
 * Free configuration handle.
 * 
 * @param config Configuration handle to free
 */
void maidos_config_free(MaidosConfig* config);

/* ============================================================================
 * maidos-auth
 * ============================================================================ */

/** Capability flags */
typedef enum {
    MAIDOS_CAP_LLM_CHAT = 1 << 0,
    MAIDOS_CAP_LLM_COMPLETE = 1 << 1,
    MAIDOS_CAP_LLM_EMBED = 1 << 2,
    MAIDOS_CAP_LLM_VISION = 1 << 3,
    MAIDOS_CAP_LLM_FUNCTION = 1 << 4,
    MAIDOS_CAP_CONFIG_READ = 1 << 5,
    MAIDOS_CAP_CONFIG_WRITE = 1 << 6,
    MAIDOS_CAP_BUS_PUBLISH = 1 << 7,
    MAIDOS_CAP_BUS_SUBSCRIBE = 1 << 8,
    MAIDOS_CAP_AUTH_ISSUE = 1 << 9,
    MAIDOS_CAP_AUTH_REVOKE = 1 << 10,
    MAIDOS_CAP_ADMIN = 1 << 15
} MaidosCapability;

/** Opaque token issuer handle */
typedef struct MaidosTokenIssuer MaidosTokenIssuer;

/**
 * Create a token issuer.
 * 
 * @param secret Secret key for signing
 * @param secret_len Length of secret key
 * @param ttl_secs Token time-to-live in seconds
 * @return Token issuer handle, or NULL on error
 */
MaidosTokenIssuer* maidos_auth_issuer_create(
    const uint8_t* secret,
    size_t secret_len,
    uint64_t ttl_secs
);

/**
 * Issue a new token.
 * 
 * @param issuer Token issuer handle
 * @param capabilities Bitmask of capabilities to grant
 * @return Token string, or NULL on error
 * 
 * @note Returned string must be freed with maidos_string_free()
 */
const char* maidos_auth_issue(MaidosTokenIssuer* issuer, uint32_t capabilities);

/**
 * Verify a token and get its capabilities.
 * 
 * @param issuer Token issuer handle
 * @param token Token string to verify
 * @param out_caps Output: granted capabilities bitmask
 * @return MAIDOS_OK if valid, error code otherwise
 */
MaidosResult maidos_auth_verify(
    MaidosTokenIssuer* issuer,
    const char* token,
    uint32_t* out_caps
);

/**
 * Check if a token has a specific capability.
 * 
 * @param issuer Token issuer handle
 * @param token Token string
 * @param capability Capability to check
 * @return true if token has the capability
 */
bool maidos_auth_has_capability(
    MaidosTokenIssuer* issuer,
    const char* token,
    MaidosCapability capability
);

/**
 * Parse capability from name string.
 * 
 * @param name Capability name (e.g., "llm.chat")
 * @return Capability value, or 0 if invalid
 */
MaidosCapability maidos_auth_capability_from_name(const char* name);

/**
 * Free token issuer handle.
 * 
 * @param issuer Token issuer handle to free
 */
void maidos_auth_issuer_free(MaidosTokenIssuer* issuer);

/* ============================================================================
 * maidos-bus
 * ============================================================================ */

/** Opaque publisher handle */
typedef struct MaidosBusPublisher MaidosBusPublisher;

/** Opaque subscriber handle */
typedef struct MaidosBusSubscriber MaidosBusSubscriber;

/** Event structure */
typedef struct {
    const char* id;
    const char* topic;
    const char* source;
    uint64_t timestamp;
    const uint8_t* data;
    size_t data_len;
} MaidosBusEvent;

/**
 * Create a publisher bound to an address.
 * 
 * @param address ZeroMQ address (e.g., "tcp://127.0.0.1:5555")
 * @return Publisher handle, or NULL on error
 */
MaidosBusPublisher* maidos_bus_publisher_create(const char* address);

/**
 * Start the publisher.
 * 
 * @param publisher Publisher handle
 * @return MAIDOS_OK on success
 */
MaidosResult maidos_bus_publisher_start(MaidosBusPublisher* publisher);

/**
 * Publish an event.
 * 
 * @param publisher Publisher handle
 * @param topic Event topic
 * @param source Event source identifier
 * @param data Event payload
 * @param data_len Payload length
 * @return MAIDOS_OK on success
 */
MaidosResult maidos_bus_publish(
    MaidosBusPublisher* publisher,
    const char* topic,
    const char* source,
    const uint8_t* data,
    size_t data_len
);

/**
 * Get the bound address of the publisher.
 * 
 * @param publisher Publisher handle
 * @return Address string, or NULL on error
 * 
 * @note Returned string must be freed with maidos_string_free()
 */
const char* maidos_bus_publisher_address(MaidosBusPublisher* publisher);

/**
 * Stop the publisher.
 * 
 * @param publisher Publisher handle
 * @return MAIDOS_OK on success
 */
MaidosResult maidos_bus_publisher_stop(MaidosBusPublisher* publisher);

/**
 * Free publisher handle.
 * 
 * @param publisher Publisher handle to free
 */
void maidos_bus_publisher_free(MaidosBusPublisher* publisher);

/**
 * Create a subscriber connected to an address.
 * 
 * @param address ZeroMQ address to connect to
 * @return Subscriber handle, or NULL on error
 */
MaidosBusSubscriber* maidos_bus_subscriber_create(const char* address);

/**
 * Subscribe to a topic pattern.
 * 
 * @param subscriber Subscriber handle
 * @param pattern Topic pattern (supports wildcards)
 * @return MAIDOS_OK on success
 */
MaidosResult maidos_bus_subscribe(MaidosBusSubscriber* subscriber, const char* pattern);

/**
 * Start the subscriber.
 * 
 * @param subscriber Subscriber handle
 * @return MAIDOS_OK on success
 */
MaidosResult maidos_bus_subscriber_start(MaidosBusSubscriber* subscriber);

/**
 * Receive the next event (blocking).
 * 
 * @param subscriber Subscriber handle
 * @param timeout_ms Timeout in milliseconds (-1 for infinite)
 * @param out_event Output: received event
 * @return MAIDOS_OK on success, MAIDOS_ERR_NOT_FOUND on timeout
 */
MaidosResult maidos_bus_recv(
    MaidosBusSubscriber* subscriber,
    int32_t timeout_ms,
    MaidosBusEvent* out_event
);

/**
 * Free subscriber handle.
 * 
 * @param subscriber Subscriber handle to free
 */
void maidos_bus_subscriber_free(MaidosBusSubscriber* subscriber);

/**
 * Free an event's internal data.
 * 
 * @param event Event to free
 */
void maidos_bus_event_free(MaidosBusEvent* event);

/* ============================================================================
 * maidos-llm
 * ============================================================================ */

/** LLM Provider type */
typedef enum {
    MAIDOS_LLM_OPENAI = 0,
    MAIDOS_LLM_ANTHROPIC = 1,
    MAIDOS_LLM_GOOGLE = 2,
    MAIDOS_LLM_DEEPSEEK = 3,
    MAIDOS_LLM_GROQ = 4,
    MAIDOS_LLM_OLLAMA = 10,
    MAIDOS_LLM_LMSTUDIO = 11,
    MAIDOS_LLM_VLLM = 12
} MaidosLlmProviderType;

/** Opaque LLM provider handle */
typedef struct MaidosLlmProvider MaidosLlmProvider;

/** Completion response */
typedef struct {
    const char* text;
    const char* model;
    uint32_t prompt_tokens;
    uint32_t completion_tokens;
    uint32_t total_tokens;
    const char* finish_reason;
} MaidosLlmResponse;

/**
 * Create an LLM provider.
 * 
 * @param provider Provider name ("openai", "anthropic", "ollama", etc.)
 * @param api_key API key (NULL for local providers)
 * @param base_url Custom base URL (NULL for default)
 * @return Provider handle, or NULL on error
 */
MaidosLlmProvider* maidos_llm_create(
    const char* provider,
    const char* api_key,
    const char* base_url
);

/**
 * Create an LLM provider by type.
 * 
 * @param provider_type Provider type enum
 * @param api_key API key (NULL for local providers)
 * @param base_url Custom base URL (NULL for default)
 * @return Provider handle, or NULL on error
 */
MaidosLlmProvider* maidos_llm_create_by_type(
    MaidosLlmProviderType provider_type,
    const char* api_key,
    const char* base_url
);

/**
 * Get provider name.
 * 
 * @param provider Provider handle
 * @return Provider name string
 */
const char* maidos_llm_provider_name(const MaidosLlmProvider* provider);

/**
 * Get provider default model.
 * 
 * @param provider Provider handle
 * @return Default model string
 */
const char* maidos_llm_default_model(const MaidosLlmProvider* provider);

/**
 * Simple completion request.
 * 
 * @param provider Provider handle
 * @param prompt User prompt
 * @param out_response Output: completion response
 * @return MAIDOS_OK on success
 * 
 * @note Response must be freed with maidos_llm_response_free()
 */
MaidosResult maidos_llm_complete(
    MaidosLlmProvider* provider,
    const char* prompt,
    MaidosLlmResponse* out_response
);

/**
 * Completion with model override.
 * 
 * @param provider Provider handle
 * @param prompt User prompt
 * @param model Model to use
 * @param out_response Output: completion response
 * @return MAIDOS_OK on success
 */
MaidosResult maidos_llm_complete_with_model(
    MaidosLlmProvider* provider,
    const char* prompt,
    const char* model,
    MaidosLlmResponse* out_response
);

/**
 * Completion with full options (JSON request).
 * 
 * @param provider Provider handle
 * @param request_json JSON request string
 * @param out_response Output: completion response
 * @return MAIDOS_OK on success
 */
MaidosResult maidos_llm_complete_json(
    MaidosLlmProvider* provider,
    const char* request_json,
    MaidosLlmResponse* out_response
);

/**
 * Free completion response.
 * 
 * @param response Response to free
 */
void maidos_llm_response_free(MaidosLlmResponse* response);

/**
 * Free LLM provider handle.
 * 
 * @param provider Provider handle to free
 */
void maidos_llm_free(MaidosLlmProvider* provider);

/* ============================================================================
 * Utility Functions
 * ============================================================================ */

/**
 * Free a string allocated by MAIDOS functions.
 * 
 * @param s String to free
 */
void maidos_string_free(const char* s);

/**
 * Get the last error message.
 * 
 * @return Error message, or NULL if no error
 * 
 * @note Returned string is valid until next MAIDOS call
 */
const char* maidos_last_error(void);

/**
 * Get library version.
 * 
 * @return Version string (e.g., "0.1.0")
 */
const char* maidos_version(void);

#ifdef __cplusplus
}
#endif

#endif /* MAIDOS_H */
