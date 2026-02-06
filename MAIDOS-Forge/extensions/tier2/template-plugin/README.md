# Template Language Plugin

This is a template for creating new language plugins for MAIDOS Forge.

## Getting Started

1. Copy this template directory and rename it to your language name
2. Update the `manifest.json` with your language information
3. Implement the `LanguageAdapter` trait in `src/lib.rs`
4. Add your language-specific compilation and parsing logic
5. Write tests to ensure everything works correctly

## Implementation Guide

The main implementation should be in `src/lib.rs`. You need to implement all methods of the `LanguageAdapter` trait:

```rust
use maidos_forge_core::languages::adapter::LanguageAdapter;

pub struct MyLanguageAdapter;

impl LanguageAdapter for MyLanguageAdapter {
    // Implement all required methods
}
```

## Testing

Add your tests in the `tests/` directory. Integration tests are recommended to ensure your plugin works correctly with the MAIDOS Forge core.

## Distribution

Once you've implemented and tested your plugin, you can distribute it by:
1. Publishing it to the MAIDOS Plugin Registry
2. Sharing the package directly with users
3. Submitting it to be included in the official extensions repository