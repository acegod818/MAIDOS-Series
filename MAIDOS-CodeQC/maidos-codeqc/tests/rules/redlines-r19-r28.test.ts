/**
 * Redlines R19-R28 Tests
 *
 * R19 固定字串回傳 | R20 模板複製灌水 | R21 假認證
 * R22 資料量不足   | R23 永遠失敗     | R24 自白註解
 * R25 日誌灌水     | R26 幽靈異步     | R27 錯誤洗白
 * R28 複製軍團
 */

import { describe, it, expect } from 'vitest';
import { checkRedlines } from '../../src/rules/b-redlines.js';

// =============================================================================
// R19: 固定字串回傳
// =============================================================================

describe('R19: 固定字串回傳', () => {
  it('should detect Rust Ok(Some("hardcoded".to_string()))', () => {
    const code = `Ok(Some("Rust interface definition".to_string()))`;
    const violations = checkRedlines(code, 'src/plugin.rs');
    expect(violations.some(v => v.ruleId === 'R19')).toBe(true);
  });

  it('should detect hardcoded confidence/score value', () => {
    const code = `confidence: 0.95`;
    const violations = checkRedlines(code, 'src/analyzer.ts');
    expect(violations.some(v => v.ruleId === 'R19')).toBe(true);
  });

  it('should skip test files', () => {
    const code = `Ok(Some("test result".to_string()))`;
    const violations = checkRedlines(code, 'tests/test_plugin.rs');
    expect(violations.filter(v => v.ruleId === 'R19')).toHaveLength(0);
  });
});

// =============================================================================
// R20: 模板複製灌水
// =============================================================================

describe('R20: 模板複製灌水', () => {
  it('should detect SupportsInterfaceExtraction = false', () => {
    const code = `SupportsInterfaceExtraction = false`;
    const violations = checkRedlines(code, 'src/Plugin.cs');
    expect(violations.some(v => v.ruleId === 'R20')).toBe(true);
  });

  it('should detect function returning Array.Empty', () => {
    const code = `ExtractSymbolsAsync(string path) { return Array.Empty<string>(); }`;
    const violations = checkRedlines(code, 'src/Plugin.cs');
    expect(violations.some(v => v.ruleId === 'R20')).toBe(true);
  });

  it('should skip test files', () => {
    const code = `SupportsGlueGeneration = false`;
    const violations = checkRedlines(code, 'tests/PluginTest.cs');
    expect(violations.filter(v => v.ruleId === 'R20')).toHaveLength(0);
  });
});

// =============================================================================
// R21: 假認證
// =============================================================================

describe('R21: 假認證', () => {
  it('should detect let _ = suppressing credential usage warning', () => {
    const code = `let _ = (&self.consumer_key, &self.consumer_secret);`;
    const violations = checkRedlines(code, 'src/auth.rs');
    expect(violations.some(v => v.ruleId === 'R21')).toBe(true);
  });

  it('should skip test files', () => {
    const code = `let _ = (&self.consumer_key, &self.consumer_secret);`;
    const violations = checkRedlines(code, 'tests/auth_test.rs');
    expect(violations.filter(v => v.ruleId === 'R21')).toHaveLength(0);
  });
});

// =============================================================================
// R22: 資料量不足
// =============================================================================

describe('R22: 資料量不足', () => {
  it('should detect small JSON dictionary', () => {
    // Simulate a dictionary JSON with very few entries
    const entries = Array(50).fill('{"char": "x", "reading": "y"}').join(',\n');
    const code = `[${entries}]`;
    const violations = checkRedlines(code, 'data/dict_table.json');
    expect(violations.some(v => v.ruleId === 'R22')).toBe(true);
  });

  it('should skip non-dictionary files', () => {
    const code = `{"char": "test"}`;
    const violations = checkRedlines(code, 'src/config.json');
    expect(violations.filter(v => v.ruleId === 'R22')).toHaveLength(0);
  });
});

// =============================================================================
// R23: 永遠失敗
// =============================================================================

describe('R23: 永遠失敗', () => {
  it('should detect fn that always returns Err', () => {
    const code = `fn verify_signature(data: &[u8]) -> Result<bool> { Err("not supported".into()) }`;
    const violations = checkRedlines(code, 'src/crypto.rs');
    expect(violations.some(v => v.ruleId === 'R23')).toBe(true);
  });

  it('should detect supports_x always returning false', () => {
    const code = `pub fn supports_encryption(&self) -> bool { false }`;
    const violations = checkRedlines(code, 'src/plugin.rs');
    expect(violations.some(v => v.ruleId === 'R23')).toBe(true);
  });

  it('should detect C# SupportsX => false', () => {
    const code = `SupportsEncryption => false`;
    const violations = checkRedlines(code, 'src/Plugin.cs');
    expect(violations.some(v => v.ruleId === 'R23')).toBe(true);
  });

  it('should skip test files', () => {
    const code = `fn supports_mock(&self) -> bool { false }`;
    const violations = checkRedlines(code, 'tests/test_plugin.rs');
    expect(violations.filter(v => v.ruleId === 'R23')).toHaveLength(0);
  });
});

// =============================================================================
// R24: 自白註解
// =============================================================================

describe('R24: 自白註解', () => {
  it('should detect "In a real implementation" comment', () => {
    const code = `// In a real implementation, this would call the API`;
    const violations = checkRedlines(code, 'src/service.ts');
    expect(violations.some(v => v.ruleId === 'R24')).toBe(true);
  });

  it('should detect "simplified implementation" comment', () => {
    const code = `// This is a simplified implementation`;
    const violations = checkRedlines(code, 'src/parser.rs');
    expect(violations.some(v => v.ruleId === 'R24')).toBe(true);
  });

  it('should detect "Placeholder" comment', () => {
    const code = `// Placeholder until real logic is added`;
    const violations = checkRedlines(code, 'src/handler.ts');
    expect(violations.some(v => v.ruleId === 'R24')).toBe(true);
  });

  it('should skip test files', () => {
    const code = `// In a real implementation, we would verify the token`;
    const violations = checkRedlines(code, 'tests/service.test.ts');
    expect(violations.filter(v => v.ruleId === 'R24')).toHaveLength(0);
  });
});

// =============================================================================
// R25: 日誌灌水
// =============================================================================

describe('R25: 日誌灌水', () => {
  it('should detect Rust fn with only log + Ok(())', () => {
    const code = [
      'pub fn backup_drivers(&self) -> Result<()> {',
      '    log::info!("Starting backup");',
      '    Ok(())',
      '}',
    ].join('\n');
    const violations = checkRedlines(code, 'src/backup.rs');
    expect(violations.some(v => v.ruleId === 'R25')).toBe(true);
  });

  it('should not flag function with real logic', () => {
    const code = [
      'pub fn backup_drivers(&self) -> Result<()> {',
      '    log::info!("Starting backup");',
      '    let drivers = self.scan_all()?;',
      '    self.write_backup(drivers)?;',
      '    Ok(())',
      '}',
    ].join('\n');
    const violations = checkRedlines(code, 'src/backup.rs');
    expect(violations.filter(v => v.ruleId === 'R25')).toHaveLength(0);
  });

  it('should skip test files', () => {
    const code = [
      'pub fn mock_backup(&self) -> Result<()> {',
      '    log::info!("mock backup");',
      '    Ok(())',
      '}',
    ].join('\n');
    const violations = checkRedlines(code, 'tests/backup_test.rs');
    expect(violations.filter(v => v.ruleId === 'R25')).toHaveLength(0);
  });
});

// =============================================================================
// R26: 幽靈異步
// =============================================================================

describe('R26: 幽靈異步', () => {
  it('should detect async fn without .await (Rust)', () => {
    const code = [
      'pub async fn download_driver(&self, url: &str) -> Result<()> {',
      '    let path = format!("/tmp/{}", url);',
      '    Ok(())',
      '}',
    ].join('\n');
    const violations = checkRedlines(code, 'src/downloader.rs');
    expect(violations.some(v => v.ruleId === 'R26')).toBe(true);
  });

  it('should not flag async fn with .await', () => {
    const code = [
      'pub async fn download_driver(&self, url: &str) -> Result<()> {',
      '    let resp = reqwest::get(url).await?;',
      '    let bytes = resp.bytes().await?;',
      '    Ok(())',
      '}',
    ].join('\n');
    const violations = checkRedlines(code, 'src/downloader.rs');
    expect(violations.filter(v => v.ruleId === 'R26')).toHaveLength(0);
  });

  it('should detect C# async method without await', () => {
    const code = [
      'public async Task<bool> ValidateAsync(string input) {',
      '    return input.Length > 0;',
      '}',
    ].join('\n');
    const violations = checkRedlines(code, 'src/Validator.cs');
    expect(violations.some(v => v.ruleId === 'R26')).toBe(true);
  });

  it('should skip non-Rust/C# files', () => {
    const code = `async function download() { return true; }`;
    const violations = checkRedlines(code, 'src/download.ts');
    expect(violations.filter(v => v.ruleId === 'R26')).toHaveLength(0);
  });
});

// =============================================================================
// R27: 錯誤洗白
// =============================================================================

describe('R27: 錯誤洗白', () => {
  it('should detect unwrap_or_default in security function (Rust)', () => {
    const code = [
      'pub fn verify_signature(&self, data: &[u8]) -> Vec<u8> {',
      '    let key = hex::decode(&self.private_key).unwrap_or_default();',
      '    key',
      '}',
    ].join('\n');
    const violations = checkRedlines(code, 'src/crypto.rs');
    expect(violations.some(v => v.ruleId === 'R27')).toBe(true);
  });

  it('should not flag unwrap_or_default in non-Rust files', () => {
    const code = `const val = map.get("key").unwrap_or_default();`;
    const violations = checkRedlines(code, 'src/util.ts');
    expect(violations.filter(v => v.ruleId === 'R27')).toHaveLength(0);
  });

  it('should skip test files', () => {
    const code = [
      'fn test_verify() {',
      '    let key = hex::decode("abc").unwrap_or_default();',
      '}',
    ].join('\n');
    const violations = checkRedlines(code, 'tests/crypto_test.rs');
    expect(violations.filter(v => v.ruleId === 'R27')).toHaveLength(0);
  });
});

// =============================================================================
// R28: 複製軍團
// =============================================================================

describe('R28: 複製軍團', () => {
  it('should detect TcpListener::bind("127.0.0.1:0") in production code', () => {
    const code = `let listener = TcpListener::bind("127.0.0.1:0").unwrap();`;
    const violations = checkRedlines(code, 'src/server.rs');
    expect(violations.some(v => v.ruleId === 'R28')).toBe(true);
  });

  it('should skip test files', () => {
    const code = `let listener = TcpListener::bind("127.0.0.1:0").unwrap();`;
    const violations = checkRedlines(code, 'tests/server_test.rs');
    expect(violations.filter(v => v.ruleId === 'R28')).toHaveLength(0);
  });
});
