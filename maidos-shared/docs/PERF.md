# Performance — maidos-shared

## Benchmark Suite

Benchmarks use Criterion.rs with HTML reports.

| Benchmark | Target | Description |
|-----------|--------|-------------|
| auth_bench | < 1ms token validation | JWT/HMAC verification throughput |
| bus_bench | < 100us per message | Event bus publish/subscribe latency |
| config_bench | < 5ms reload | Config file parse and hot-reload |
| llm_bench | < 50ms first token | LLM provider connection and first response |
| ffi_bench | < 10us per call | FFI boundary crossing overhead |

## Running Benchmarks

```
cargo bench --workspace
cargo bench -p maidos-auth
```

## Optimization Guidelines

- Use parking_lot over std::sync for lower contention
- Prefer bytes::Bytes for zero-copy buffer sharing
- Use rmp-serde (MessagePack) for internal serialization, JSON only at API boundary
- Async operations via tokio with full feature set
