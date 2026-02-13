/**
 * v3.4 R19-R24 驗證腳本
 * 
 * 用這次審計發現的 20 種真實假實作原文當測試案例
 * 證明新規則能抓到 v3.3 漏掉的那 14 種手法
 */

import { R19_CHECKER, R20_CHECKER, R21_CHECKER, R22_CHECKER, R23_CHECKER, R24_CHECKER } from './src/rules/b-redlines-r19-r24.js';
import { R13_CHECKER, R14_CHECKER, R15_CHECKER, R16_CHECKER, R17_CHECKER, R18_CHECKER } from './src/rules/b-redlines-r13-r18.js';

interface TestCase {
  name: string;
  source: string;
  file: string;
  /** v3.3 R13-R18 能抓到嗎 */
  v33catches: boolean;
  /** v3.4 R19-R24 預期抓到的規則 */
  expectedRule: string;
}

const AUDIT_CASES: TestCase[] = [
  // === R19 固定字串回傳 ===
  {
    name: '假 interface 描述 (Forge-core rust.rs)',
    source: `    async fn extract_interface(&self, artifact: &str) -> Result<Option<String>> {
        Ok(Some("Rust interface definition".to_string()))
    }`,
    file: 'src/languages/rust.rs',
    v33catches: false,
    expectedRule: 'R19',
  },
  {
    name: '假 confidence (HandwritingInput.cs)',
    source: `    private float ParseConfidence(string output) {
        return Math.Min(0.95f, output.Length / 20.0f);
    }`,
    file: 'src/HandwritingInput.cs',
    v33catches: false,
    expectedRule: 'R19',
  },

  // === R20 模板複製灌水 ===
  {
    name: '空殼 plugin - SupportsInterfaceExtraction=false (84個)',
    source: `    public PluginCapabilities GetCapabilities() => new() {
        LanguageName = "zig",
        SupportsInterfaceExtraction = false,
        SupportsGlueGeneration = false,
    };`,
    file: 'Forge.Plugin.Zig/ZigPlugin.cs',
    v33catches: false,
    expectedRule: 'R20',
  },

  // === R21 假認證 ===
  {
    name: '壓 unused warning 的 OAuth (twitter.rs)',
    source: `    fn build_bearer_token(&self) -> String {
        let _ = (&self.consumer_key, &self.consumer_secret, &self.access_token_secret);
        format!("Bearer {}", self.access_token)
    }`,
    file: 'src/twitter.rs',
    v33catches: false,
    expectedRule: 'R21',
  },

  // === R22 資料量不足 ===
  {
    name: '五筆字典僅 247 條 (wubi_table.json)',
    source: Array.from({ length: 247 }, (_, i) =>
      `  { "code": "code${i}", "char": "字", "freq": ${1000 - i} }`
    ).join(',\n'),
    file: 'src/core/data/wubi_table.json',
    v33catches: false,
    expectedRule: 'R22',
  },

  // === R23 永遠失敗 ===
  {
    name: 'HandwritingScheme 永遠 Err (schemes.rs)',
    source: `    fn process_input(&self, _input: &str) -> Result<Vec<Candidate>> {
        Err(MaidosError::SchemeError(
            "Handwriting requires platform-specific integration".to_string()
        ))
    }`,
    file: 'src/schemes.rs',
    v33catches: false,
    expectedRule: 'R23',
  },
  {
    name: 'supports_tools 永遠 false',
    source: `    pub fn supports_tools(&self) -> bool { false }`,
    file: 'src/providers/replicate.rs',
    v33catches: false,
    expectedRule: 'R23',
  },

  // === R24 自白註解 ===
  {
    name: '"In a real implementation" 註解 (google contacts.rs)',
    source: `    async fn refresh_auth(&self) -> Result<()> {
        // In a real implementation, this would refresh the OAuth2 token
        Ok(())
    }`,
    file: 'src/contacts.rs',
    v33catches: false,
    expectedRule: 'R24',
  },
  {
    name: '"simplified implementation" 註解 (plugin.rs)',
    source: `    // This is a simplified implementation; the actual one varies by language
    let result = CompileResult::default();`,
    file: 'src/plugin.rs',
    v33catches: false,
    expectedRule: 'R24',
  },
  {
    name: '"For simplicity" 註解 (config.rs)',
    source: `    // For simplicity, we return the default configuration
    Ok(Self::default())`,
    file: 'src/config.rs',
    v33catches: false,
    expectedRule: 'R24',
  },
  {
    name: '"實際實現中" 中文自白 (Faust plugin)',
    source: `    Exports = (await NativeSymbolExtractor.ExtractFromBinaryAsync(artifactPath, "faust", ct)).ToArray(), // 實際實現中需要解析源碼`,
    file: 'Forge.Plugin.Faust/FaustPlugin.cs',
    v33catches: false,
    expectedRule: 'R24',
  },
  {
    name: '"目前使用簡化" (backup_manager.rs)',
    source: `                // 目前使用簡化的實現
    let backup_info = BackupInfo {
        device_id: "unknown".to_string(),
    };`,
    file: 'src/backup_manager.rs',
    v33catches: false,
    expectedRule: 'R24',
  },

  // === 驗證 v3.3 能抓到的（不應退化） ===
  {
    name: 'todo! (v3.3 R13 能抓)',
    source: `    fn process(&self) { todo!() }`,
    file: 'src/main.rs',
    v33catches: true,
    expectedRule: 'R13',
  },
  {
    name: '空 catch (v3.3 R14 能抓)',
    source: `    try { riskyOp(); } catch (e) {}`,
    file: 'src/handler.ts',
    v33catches: true,
    expectedRule: 'R14',
  },
  {
    name: '// TODO (v3.3 R15 能抓)',
    source: `    // TODO: implement real logic`,
    file: 'src/service.ts',
    v33catches: true,
    expectedRule: 'R15',
  },
];

// === 執行測試 ===
const V33_CHECKERS = [R13_CHECKER, R14_CHECKER, R15_CHECKER, R16_CHECKER, R17_CHECKER, R18_CHECKER];
const V34_CHECKERS = [R19_CHECKER, R20_CHECKER, R21_CHECKER, R22_CHECKER, R23_CHECKER, R24_CHECKER];
const ALL_CHECKERS = [...V33_CHECKERS, ...V34_CHECKERS];

let passed = 0;
let failed = 0;
const results: string[] = [];

for (const tc of AUDIT_CASES) {
  // Check if ANY checker catches it
  const v33Violations = V33_CHECKERS.flatMap(c => c.checkSource?.(tc.source, tc.file) ?? []);
  const v34Violations = V34_CHECKERS.flatMap(c => c.checkSource?.(tc.source, tc.file) ?? []);
  const allViolations = [...v33Violations, ...v34Violations];

  const caughtByExpected = allViolations.some(v => v.ruleId === tc.expectedRule);
  const caughtByAnything = allViolations.length > 0;
  const v33Caught = v33Violations.length > 0;

  let status: string;
  if (caughtByExpected) {
    status = '✅ PASS';
    passed++;
  } else if (caughtByAnything) {
    status = `⚠️ CAUGHT by ${allViolations[0]!.ruleId} (expected ${tc.expectedRule})`;
    passed++; // Still caught, just different rule
  } else {
    status = `❌ MISS — expected ${tc.expectedRule}`;
    failed++;
  }

  const v33Status = tc.v33catches ? (v33Caught ? '✅' : '❌') : (v33Caught ? '🎁bonus' : '—');

  results.push(`${status} | v3.3=${v33Status} | ${tc.name}`);
}

console.log('╔══════════════════════════════════════════════════════════════╗');
console.log('║  Code-QC v3.4 R19-R24 驗證結果 (對審計發現的真實假實作)    ║');
console.log('╠══════════════════════════════════════════════════════════════╣');
for (const r of results) {
  console.log(`║ ${r}`);
}
console.log('╠══════════════════════════════════════════════════════════════╣');
console.log(`║ 通過: ${passed}/${AUDIT_CASES.length} | 漏殺: ${failed}/${AUDIT_CASES.length}`);
console.log(`║ v3.3 覆蓋: ${AUDIT_CASES.filter(t => t.v33catches).length}/${AUDIT_CASES.length}`);
console.log(`║ v3.4 覆蓋: ${passed}/${AUDIT_CASES.length}`);
console.log(`║ 漏殺率: v3.3=${Math.round((1 - AUDIT_CASES.filter(t => t.v33catches).length / AUDIT_CASES.length) * 100)}% → v3.4=${Math.round((failed / AUDIT_CASES.length) * 100)}%`);
console.log('╚══════════════════════════════════════════════════════════════╝');

process.exit(failed > 0 ? 1 : 0);
