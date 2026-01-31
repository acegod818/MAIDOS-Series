// <impl>
// WHAT: C# wrapper for maidos-llm native library
// WHY: Provide idiomatic C# API for LLM operations
// HOW: IDisposable for handles, CompletionResult class, fluent builder for requests
// TEST: Unit tests for provider creation, completion, error handling
// </impl>

using System.Runtime.InteropServices;

namespace MaidosShared;

/// <summary>
/// Exception thrown when an LLM operation fails.
/// </summary>
public class LlmException : Exception
{
    public LlmException(string message) : base(message) { }
}

/// <summary>
/// Supported LLM providers.
/// </summary>
public enum LlmProviderType
{
    /// <summary>OpenAI GPT models</summary>
    OpenAI,
    /// <summary>Anthropic Claude models</summary>
    Anthropic,
    /// <summary>Local Ollama models</summary>
    Ollama
}

/// <summary>
/// Result from an LLM completion request.
/// </summary>
public sealed class CompletionResult
{
    /// <summary>The generated text response.</summary>
    public string Text { get; }

    /// <summary>Number of tokens in the prompt.</summary>
    public uint PromptTokens { get; }

    /// <summary>Number of tokens in the completion.</summary>
    public uint CompletionTokens { get; }

    /// <summary>Total tokens used.</summary>
    public uint TotalTokens => PromptTokens + CompletionTokens;

    /// <summary>Model that generated the response.</summary>
    public string Model { get; }

    internal CompletionResult(string text, uint promptTokens, uint completionTokens, string model)
    {
        Text = text;
        PromptTokens = promptTokens;
        CompletionTokens = completionTokens;
        Model = model;
    }
}

/// <summary>
/// Options for LLM completion requests.
/// </summary>
public sealed class CompletionOptions
{
    /// <summary>Maximum tokens to generate (0 for default).</summary>
    public uint MaxTokens { get; set; } = 0;

    /// <summary>Temperature for sampling (negative for default).</summary>
    public float Temperature { get; set; } = -1f;

    /// <summary>System prompt to set context.</summary>
    public string? System { get; set; }

    /// <summary>Create default options.</summary>
    public static CompletionOptions Default => new();

    /// <summary>Set max tokens (fluent).</summary>
    public CompletionOptions WithMaxTokens(uint maxTokens)
    {
        MaxTokens = maxTokens;
        return this;
    }

    /// <summary>Set temperature (fluent).</summary>
    public CompletionOptions WithTemperature(float temperature)
    {
        Temperature = temperature;
        return this;
    }

    /// <summary>Set system prompt (fluent).</summary>
    public CompletionOptions WithSystem(string system)
    {
        System = system;
        return this;
    }
}

/// <summary>
/// MAIDOS LLM client.
/// Provides access to various LLM providers through a unified interface.
/// </summary>
/// <remarks>
/// Example usage:
/// <code>
/// // OpenAI
/// using var client = MaidosLlm.Create(LlmProviderType.OpenAI, "sk-xxx");
/// var result = client.Complete("gpt-4o", "Hello, how are you?");
/// Console.WriteLine(result.Text);
///
/// // Anthropic
/// using var claude = MaidosLlm.Create(LlmProviderType.Anthropic, "sk-ant-xxx");
/// var response = claude.Complete("claude-sonnet-4-20250514", "What is 2+2?",
///     new CompletionOptions().WithSystem("You are a math tutor."));
///
/// // Ollama (local, no API key)
/// using var ollama = MaidosLlm.Create(LlmProviderType.Ollama);
/// var local = ollama.Complete("llama3.2", "Explain quantum computing");
/// </code>
/// </remarks>
public sealed class MaidosLlm : IDisposable
{
    private IntPtr _handle;
    private bool _disposed;
    private readonly string _providerName;

    private MaidosLlm(IntPtr handle, string providerName)
    {
        _handle = handle;
        _providerName = providerName;
    }

    /// <summary>
    /// Create a new LLM client.
    /// </summary>
    /// <param name="provider">Provider type.</param>
    /// <param name="apiKey">API key (optional for Ollama).</param>
    /// <param name="baseUrl">Custom base URL (optional).</param>
    /// <returns>A new LLM client.</returns>
    /// <exception cref="LlmException">Thrown when provider creation fails.</exception>
    public static MaidosLlm Create(LlmProviderType provider, string? apiKey = null, string? baseUrl = null)
    {
        var providerName = provider switch
        {
            LlmProviderType.OpenAI => "openai",
            LlmProviderType.Anthropic => "anthropic",
            LlmProviderType.Ollama => "ollama",
            _ => throw new ArgumentException($"Unknown provider: {provider}")
        };

        var handle = Native.LlmCreate(providerName, apiKey, baseUrl);
        if (handle == IntPtr.Zero)
        {
            throw new LlmException($"Failed to create {provider} provider. Check API key.");
        }

        return new MaidosLlm(handle, providerName);
    }

    /// <summary>
    /// Create an OpenAI client.
    /// </summary>
    /// <param name="apiKey">OpenAI API key.</param>
    /// <param name="baseUrl">Custom base URL (for API-compatible providers).</param>
    public static MaidosLlm OpenAI(string apiKey, string? baseUrl = null)
        => Create(LlmProviderType.OpenAI, apiKey, baseUrl);

    /// <summary>
    /// Create an Anthropic (Claude) client.
    /// </summary>
    /// <param name="apiKey">Anthropic API key.</param>
    public static MaidosLlm Anthropic(string apiKey)
        => Create(LlmProviderType.Anthropic, apiKey);

    /// <summary>
    /// Create a local Ollama client.
    /// </summary>
    /// <param name="baseUrl">Ollama server URL (default: http://localhost:11434).</param>
    public static MaidosLlm Ollama(string? baseUrl = null)
        => Create(LlmProviderType.Ollama, null, baseUrl);

    /// <summary>
    /// Get the provider name.
    /// </summary>
    public string ProviderName
    {
        get
        {
            ThrowIfDisposed();
            var ptr = Native.LlmProviderName(_handle);
            if (ptr == IntPtr.Zero)
            {
                return _providerName;
            }
            try
            {
                return Marshal.PtrToStringUTF8(ptr) ?? _providerName;
            }
            finally
            {
                Native.LlmStringFree(ptr);
            }
        }
    }

    /// <summary>
    /// Check if the provider is healthy and reachable.
    /// </summary>
    public bool IsHealthy
    {
        get
        {
            ThrowIfDisposed();
            return Native.LlmHealthCheck(_handle);
        }
    }

    /// <summary>
    /// Complete a chat request.
    /// </summary>
    /// <param name="model">Model to use (e.g., "gpt-4o", "claude-sonnet-4-20250514", "llama3.2").</param>
    /// <param name="message">User message.</param>
    /// <param name="options">Completion options.</param>
    /// <returns>Completion result.</returns>
    /// <exception cref="LlmException">Thrown when completion fails.</exception>
    public CompletionResult Complete(string model, string message, CompletionOptions? options = null)
    {
        ThrowIfDisposed();
        ArgumentNullException.ThrowIfNull(model);
        ArgumentNullException.ThrowIfNull(message);

        options ??= CompletionOptions.Default;

        var resultPtr = Native.LlmComplete(
            _handle,
            model,
            options.System,
            message,
            options.MaxTokens,
            options.Temperature);

        if (resultPtr == IntPtr.Zero)
        {
            throw new LlmException("Completion failed: null result");
        }

        try
        {
            var result = Marshal.PtrToStructure<FfiCompletionResult>(resultPtr);

            if (!result.Success)
            {
                var error = result.Error != IntPtr.Zero
                    ? Marshal.PtrToStringUTF8(result.Error) ?? "Unknown error"
                    : "Completion failed";
                throw new LlmException(error);
            }

            var text = result.Text != IntPtr.Zero
                ? Marshal.PtrToStringUTF8(result.Text) ?? ""
                : "";

            var modelName = result.Model != IntPtr.Zero
                ? Marshal.PtrToStringUTF8(result.Model) ?? model
                : model;

            return new CompletionResult(text, result.PromptTokens, result.CompletionTokens, modelName);
        }
        finally
        {
            Native.LlmResultFree(resultPtr);
        }
    }

    /// <summary>
    /// Complete a chat request with a system prompt.
    /// </summary>
    /// <param name="model">Model to use.</param>
    /// <param name="system">System prompt.</param>
    /// <param name="message">User message.</param>
    /// <returns>Completion result.</returns>
    public CompletionResult Complete(string model, string system, string message)
    {
        return Complete(model, message, new CompletionOptions { System = system });
    }

    private void ThrowIfDisposed()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
    }

    /// <summary>
    /// Releases the LLM client handle.
    /// </summary>
    public void Dispose()
    {
        if (!_disposed)
        {
            if (_handle != IntPtr.Zero)
            {
                Native.LlmDestroy(_handle);
                _handle = IntPtr.Zero;
            }
            _disposed = true;
        }
    }
}
