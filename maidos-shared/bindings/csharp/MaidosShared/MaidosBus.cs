// <impl>
// WHAT: C# wrapper for maidos-bus native library
// WHY: Provide idiomatic C# API for pub/sub event bus
// HOW: IDisposable for handles, BusEvent class for events, async-friendly design
// TEST: Unit tests for publish, subscribe, event marshaling
// </impl>

using System.Runtime.InteropServices;
using System.Text;

namespace MaidosShared;

/// <summary>
/// Exception thrown when a bus operation fails.
/// </summary>
public class BusException : Exception
{
    public BusException(string message) : base(message) { }
}

/// <summary>
/// An event from the MAIDOS event bus.
/// </summary>
public sealed class BusEvent
{
    /// <summary>Topic this event was published to.</summary>
    public string Topic { get; }

    /// <summary>Unique event identifier.</summary>
    public string Id { get; }

    /// <summary>Unix timestamp in milliseconds.</summary>
    public ulong Timestamp { get; }

    /// <summary>Source that published the event.</summary>
    public string Source { get; }

    /// <summary>Event payload as bytes.</summary>
    public byte[] Payload { get; }

    internal BusEvent(string topic, string id, ulong timestamp, string source, byte[] payload)
    {
        Topic = topic;
        Id = id;
        Timestamp = timestamp;
        Source = source;
        Payload = payload;
    }

    /// <summary>Get payload as UTF-8 string.</summary>
    public string PayloadAsString() => Encoding.UTF8.GetString(Payload);

    /// <summary>Get timestamp as DateTimeOffset.</summary>
    public DateTimeOffset TimestampAsDateTime() =>
        DateTimeOffset.FromUnixTimeMilliseconds((long)Timestamp);
}

/// <summary>
/// MAIDOS event bus publisher.
/// Publishes events to connected subscribers.
/// </summary>
/// <remarks>
/// Example usage:
/// <code>
/// using var publisher = new BusPublisher("127.0.0.1:0");
/// var port = publisher.Port;
/// publisher.Publish("my.topic", "my-source", Encoding.UTF8.GetBytes("Hello!"));
/// </code>
/// </remarks>
public sealed class BusPublisher : IDisposable
{
    private IntPtr _handle;
    private bool _disposed;

    /// <summary>
    /// Create a new publisher bound to the specified address.
    /// </summary>
    /// <param name="bindAddress">Address to bind to (e.g., "127.0.0.1:9999" or "0.0.0.0:0" for auto-port).</param>
    /// <exception cref="BusException">Thrown when binding fails.</exception>
    public BusPublisher(string bindAddress = "127.0.0.1:0")
    {
        ArgumentNullException.ThrowIfNull(bindAddress);

        _handle = Native.BusPublisherCreate(bindAddress);
        if (_handle == IntPtr.Zero)
        {
            throw new BusException($"Failed to create publisher on {bindAddress}");
        }
    }

    /// <summary>
    /// Get the port the publisher is listening on.
    /// </summary>
    public ushort Port
    {
        get
        {
            ThrowIfDisposed();
            return Native.BusPublisherPort(_handle);
        }
    }

    /// <summary>
    /// Publish an event to the bus.
    /// </summary>
    /// <param name="topic">Topic to publish to (e.g., "maidos.config.changed").</param>
    /// <param name="source">Source identifier.</param>
    /// <param name="payload">Event payload.</param>
    /// <returns>True if published successfully.</returns>
    public bool Publish(string topic, string source, byte[] payload)
    {
        ThrowIfDisposed();
        ArgumentNullException.ThrowIfNull(topic);
        ArgumentNullException.ThrowIfNull(source);
        ArgumentNullException.ThrowIfNull(payload);

        unsafe
        {
            fixed (byte* payloadPtr = payload)
            {
                return Native.BusPublish(_handle, topic, source, (IntPtr)payloadPtr, (nuint)payload.Length);
            }
        }
    }

    /// <summary>
    /// Publish a string message to the bus.
    /// </summary>
    /// <param name="topic">Topic to publish to.</param>
    /// <param name="source">Source identifier.</param>
    /// <param name="message">Message to publish (will be UTF-8 encoded).</param>
    /// <returns>True if published successfully.</returns>
    public bool Publish(string topic, string source, string message)
    {
        return Publish(topic, source, Encoding.UTF8.GetBytes(message));
    }

    private void ThrowIfDisposed()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
    }

    /// <summary>
    /// Releases the publisher handle.
    /// </summary>
    public void Dispose()
    {
        if (!_disposed)
        {
            if (_handle != IntPtr.Zero)
            {
                Native.BusPublisherDestroy(_handle);
                _handle = IntPtr.Zero;
            }
            _disposed = true;
        }
    }
}

/// <summary>
/// MAIDOS event bus subscriber.
/// Receives events from a publisher.
/// </summary>
/// <remarks>
/// Example usage:
/// <code>
/// using var subscriber = new BusSubscriber("127.0.0.1:9999", "maidos.*");
/// while (true)
/// {
///     var ev = subscriber.Receive(TimeSpan.FromSeconds(1));
///     if (ev != null)
///     {
///         Console.WriteLine($"Got event: {ev.Topic}");
///     }
/// }
/// </code>
/// </remarks>
public sealed class BusSubscriber : IDisposable
{
    private IntPtr _handle;
    private bool _disposed;

    /// <summary>
    /// Create a new subscriber connected to a publisher.
    /// </summary>
    /// <param name="connectAddress">Publisher address to connect to.</param>
    /// <param name="topicFilter">Optional topic filter (supports wildcards like "maidos.*").</param>
    /// <exception cref="BusException">Thrown when connection fails.</exception>
    public BusSubscriber(string connectAddress, string? topicFilter = null)
    {
        ArgumentNullException.ThrowIfNull(connectAddress);

        _handle = Native.BusSubscriberCreate(connectAddress, topicFilter);
        if (_handle == IntPtr.Zero)
        {
            throw new BusException($"Failed to create subscriber to {connectAddress}");
        }
    }

    /// <summary>
    /// Receive the next event from the bus.
    /// </summary>
    /// <param name="timeout">Timeout for waiting.</param>
    /// <returns>The next event, or null if timeout.</returns>
    public BusEvent? Receive(TimeSpan timeout)
    {
        ThrowIfDisposed();

        var timeoutMs = (uint)Math.Min(timeout.TotalMilliseconds, uint.MaxValue);
        var eventPtr = Native.BusReceive(_handle, timeoutMs);

        if (eventPtr == IntPtr.Zero)
        {
            return null;
        }

        try
        {
            return MarshalEvent(eventPtr);
        }
        finally
        {
            Native.BusEventFree(eventPtr);
        }
    }

    /// <summary>
    /// Receive the next event from the bus with a default 1-second timeout.
    /// </summary>
    /// <returns>The next event, or null if timeout.</returns>
    public BusEvent? Receive()
    {
        return Receive(TimeSpan.FromSeconds(1));
    }

    /// <summary>
    /// Try to receive an event without blocking.
    /// </summary>
    /// <returns>The next event, or null if none available.</returns>
    public BusEvent? TryReceive()
    {
        return Receive(TimeSpan.Zero);
    }

    private static BusEvent MarshalEvent(IntPtr eventPtr)
    {
        var ffiEvent = Marshal.PtrToStructure<FfiEvent>(eventPtr);

        var topic = Marshal.PtrToStringUTF8(ffiEvent.Topic) ?? "";
        var id = Marshal.PtrToStringUTF8(ffiEvent.Id) ?? "";
        var source = Marshal.PtrToStringUTF8(ffiEvent.Source) ?? "";

        var payload = new byte[(int)ffiEvent.PayloadLen];
        if (ffiEvent.PayloadLen > 0)
        {
            Marshal.Copy(ffiEvent.Payload, payload, 0, (int)ffiEvent.PayloadLen);
        }

        return new BusEvent(topic, id, ffiEvent.Timestamp, source, payload);
    }

    private void ThrowIfDisposed()
    {
        ObjectDisposedException.ThrowIf(_disposed, this);
    }

    /// <summary>
    /// Releases the subscriber handle.
    /// </summary>
    public void Dispose()
    {
        if (!_disposed)
        {
            if (_handle != IntPtr.Zero)
            {
                Native.BusSubscriberDestroy(_handle);
                _handle = IntPtr.Zero;
            }
            _disposed = true;
        }
    }
}
