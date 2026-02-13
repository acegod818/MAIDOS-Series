/**
 * Code-QC v3.4 — R19-R24 審計補漏規則
 *
 * 來源: MAIDOS-Series 深度審計，發現 v3.3 R13-R18 漏殺率 70%
 * 這六條規則針對 LLM 生成代碼的高階偽裝手法
 *
 * R19 固定字串回傳 — 函數看似有值但全是寫死的
 * R20 模板複製灌水 — 大量檔案只改名字，功能全空
 * R21 假認證       — 收了憑證但壓 unused warning 不用
 * R22 資料量不足   — 字典/對照表遠低於實用門檻
 * R23 永遠失敗     — 函數永遠 Err/Failure，功能等於不存在
 * R24 自白註解     — 註解自己承認是假的
 */

import type { RuleChecker, Violation } from '../types.js';
import { getRedline } from './b-redlines-defs.js';
import { maskJsStringsAndComments } from './b-redlines-utils.js';

// =============================================================================
// R19: 固定字串回傳
// 手法: Ok(Some("Rust interface definition".to_string()))
//       format!("// Glue code for {}\n{}", ...)
//       confidence: 0.95 等 hardcoded magic numbers
// =============================================================================

const HARDCODED_RETURN_PATTERNS = [
  // Rust: Ok(Some("固定字串")) — 函數聲稱回傳解析結果但回寫死字串
  /Ok\(\s*Some\(\s*"[^"]{3,60}"\.to_string\(\)\s*\)\s*\)/g,
  // Rust: Ok(format!("// comment")) — 生成的"代碼"只是註解
  /Ok\(\s*format!\(\s*"\/\/[^"]*"/g,
  // C#: return "固定字串" 在非 ToString/Name/Version 方法中
  /return\s+"[A-Z][a-z]+\s+(?:interface|definition|implementation|result|output|response)\s*(?:for)?\s*[^"]*"/gi,
  // 任何語言: hardcoded confidence/score 值 (直接賦值)
  /(?:confidence|score|probability|accuracy)\s*[:=]\s*0\.\d{1,2}f?\b/gi,
  // 任何語言: hardcoded magic float 在 return/Math 中 (0.75f, 0.85f, 0.90f, 0.95f)
  /(?:return|Math\.Min|Math\.Max|math\.min)\s*\(\s*0\.(?:7[05]|8[05]|9[05])f?\b/gi,
  // hardcoded gas limit (21000 是 ETH transfer 的，但用在所有 tx 就是假的)
  /gas(?:_limit|Limit)\s*[:=]\s*(?:U256::from\()?21000/g,
];

export const R19_CHECKER: RuleChecker = {
  rule: getRedline('R19')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    if (/(?:test|spec|mock|fixture|__test__)/i.test(file)) return violations;
    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      if (/^\s*(?:\/\/|#|\/\*|\*|--|;)/.test(line)) continue;
      for (const pattern of HARDCODED_RETURN_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({
            ruleId: 'R19', ruleName: '固定字串回傳', severity: 'error',
            file, line: i + 1, column: match.index + 1,
            message: `固定字串回傳 (R19): 回傳值可能是寫死的 — ${match[0].substring(0, 50)}`,
            snippet: line.trim(),
            suggestion: '確認回傳值來自真實計算/解析/API呼叫，不是 hardcoded',
          });
          break;
        }
      }
    }
    return violations;
  },
};

// =============================================================================
// R20: 模板複製灌水
// 手法: 84 個 plugin 除語言名稱外完全相同
//       檢測方式: 掃描同目錄下多個檔案結構相似度
//       此 checker 為單檔掃描，標記可疑模板特徵
// =============================================================================

const TEMPLATE_MARKER_PATTERNS = [
  // C#: SupportsInterfaceExtraction = false + SupportsGlueGeneration = false 同時出現
  /Supports(?:Interface|Glue|Export)(?:Extraction|Generation|Analysis)\s*=\s*false/gi,
  // 任何語言: 函數名有意義但 body 只有一行 return Array.Empty / vec![] / []
  /(?:Extract|Analyze|Parse|Generate|Process)\w+\s*(?:Async)?\s*\([^)]*\)[^{]*\{\s*(?:return\s+)?(?:Array\.Empty|vec!\[\]|\[\]|new\s+\w+\[0\]|Enumerable\.Empty)/g,
];

export const R20_CHECKER: RuleChecker = {
  rule: getRedline('R20')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    if (/(?:test|spec|mock|fixture)/i.test(file)) return violations;
    for (const pattern of TEMPLATE_MARKER_PATTERNS) {
      pattern.lastIndex = 0;
      let match;
      while ((match = pattern.exec(source)) !== null) {
        const before = source.substring(0, match.index);
        const lineNum = (before.match(/\n/g) || []).length + 1;
        violations.push({
          ruleId: 'R20', ruleName: '模板複製灌水', severity: 'error',
          file, line: lineNum, column: 1,
          message: `模板複製灌水 (R20): 功能宣告存在但核心返回空 — ${match[0].substring(0, 50)}`,
          snippet: match[0].substring(0, 80),
          suggestion: '實作真實邏輯或將 SupportsX 改為 false 並移除空殼方法',
        });
      }
    }
    return violations;
  },
};

// =============================================================================
// R21: 假認證
// 手法: let _ = (&self.consumer_key, &self.consumer_secret) 壓 unused warning
//       收了 4 個 OAuth 參數只用 1 個
// =============================================================================

const FAKE_AUTH_PATTERNS = [
  // Rust: let _ = (&self.xxx_key, ...) — 用 let _ 壓掉憑證未使用的 warning
  /let\s+_\s*=\s*\(&?self\.\w*(?:key|secret|token|password|credential|cert)\w*/gi,
  // 任何語言: 收了 secret/key 參數但函數體沒有 hmac/sign/hash/encrypt/verify
  // (此 pattern 標記可疑，需人工確認)
  /(?:consumer_secret|client_secret|api_secret|private_key)\s*[,)]/g,
];

export const R21_CHECKER: RuleChecker = {
  rule: getRedline('R21')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    if (/(?:test|spec|mock|fixture)/i.test(file)) return violations;

    // 重點掃描: let _ = 壓憑證 warning
    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      const letUnderscoreMatch = /let\s+_\s*=\s*\(?&?self\.\w*(?:key|secret|token|password|credential|cert)\w*/i.exec(line);
      if (letUnderscoreMatch) {
        violations.push({
          ruleId: 'R21', ruleName: '假認證', severity: 'error',
          file, line: i + 1, column: letUnderscoreMatch.index + 1,
          message: `假認證 (R21): 用 let _ 壓掉憑證未使用警告 — ${letUnderscoreMatch[0].substring(0, 50)}`,
          snippet: line.trim(),
          suggestion: '憑證參數必須用於真實簽名/加密/驗證，不能 let _ 丟棄',
        });
      }
    }
    return violations;
  },
};

// =============================================================================
// R22: 資料量不足
// 手法: 字典 247 條 vs 需要 6000+, 簡繁對照 2 條 vs 需要 2500+
//       掃描 JSON/TSV 資料檔的條目數量
// =============================================================================

export const R22_CHECKER: RuleChecker = {
  rule: getRedline('R22')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    if (/(?:test|spec|mock|fixture)/i.test(file)) return violations;

    // 針對 JSON 字典檔: 計算 "char" 出現次數
    if (/(?:table|dict|mapping|vocab|lexicon).*\.json$/i.test(file)) {
      const charEntries = (source.match(/"char"/g) || []).length;
      if (charEntries > 0 && charEntries < 500) {
        violations.push({
          ruleId: 'R22', ruleName: '資料量不足', severity: 'error',
          file, line: 1, column: 1,
          message: `資料量不足 (R22): 字典僅 ${charEntries} 條目，實用門檻至少 500+`,
          snippet: `${charEntries} entries in ${file.split('/').pop()}`,
          suggestion: '擴充字典到實用規模，參考 Unihan/CC-CEDICT 等開源數據',
        });
      }
    }

    // 針對 TSV 資料檔: 檢查空欄位
    if (/\.tsv$/i.test(file)) {
      const lines = source.split('\n').filter(l => l.trim() && !l.startsWith('#'));
      let emptyFields = 0;
      for (const line of lines) {
        const fields = line.split('\t');
        emptyFields += fields.filter(f => f.trim() === '').length;
      }
      if (emptyFields > 5) {
        violations.push({
          ruleId: 'R22', ruleName: '資料量不足', severity: 'error',
          file, line: 1, column: 1,
          message: `資料量不足 (R22): TSV 有 ${emptyFields} 個空欄位`,
          snippet: `${emptyFields} empty fields in ${file.split('/').pop()}`,
          suggestion: '填入真實數據或標記 VERIFY_ON_DOWNLOAD',
        });
      }
    }
    return violations;
  },
};

// =============================================================================
// R23: 永遠失敗
// 手法: 所有方法都回 Err("not supported") — 功能等於不存在
//       fn supports_x() { false } — 能力檢查永遠否定
// =============================================================================

const ALWAYS_FAIL_PATTERNS = [
  // Rust: 函數只有一行 Err(...)
  /fn\s+\w+\s*(?:<[^>]*>)?\s*\([^)]*\)\s*(?:->.*?)?\{\s*Err\(/g,
  // C#: 方法只回 .Failure(
  /\w+\s*\([^)]*\)\s*(?:=>|{)\s*(?:return\s+)?\w+\.Failure\(/g,
  // 任何語言: supports_X / has_X / can_X 永遠回 false
  /(?:pub\s+)?fn\s+(?:supports?|has|can|is)_\w+\s*\([^)]*\)\s*(?:->.*?)?\{\s*false\s*\}/g,
  // C# 版: SupportsX => false (property)
  /Supports\w+\s*(?:=>|{\s*get\s*{\s*return)\s*false/g,
];

export const R23_CHECKER: RuleChecker = {
  rule: getRedline('R23')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    if (/(?:test|spec|mock|fixture)/i.test(file)) return violations;
    const normalized = source.replace(/\r\n/g, '\n');
    for (const pattern of ALWAYS_FAIL_PATTERNS) {
      pattern.lastIndex = 0;
      let match;
      while ((match = pattern.exec(normalized)) !== null) {
        const before = normalized.substring(0, match.index);
        const lineNum = (before.match(/\n/g) || []).length + 1;
        violations.push({
          ruleId: 'R23', ruleName: '永遠失敗', severity: 'error',
          file, line: lineNum, column: 1,
          message: `永遠失敗 (R23): 函數永遠回失敗/false — ${match[0].substring(0, 50)}`,
          snippet: match[0].substring(0, 80),
          suggestion: '實作真實邏輯或明確標記為 #[cfg(not(feature = "X"))]',
        });
      }
    }
    return violations;
  },
};

// =============================================================================
// R24: 自白註解
// 手法: "In a real implementation, this would..."
//       "simplified implementation" / "for simplicity" / "for now"
//       "實際實現中" / "簡化實現" / "目前使用簡化"
//       這些註解承認代碼是假的但繼續假
// =============================================================================

const SELF_CONFESSING_PATTERNS = [
  // English
  /\/\/\s*(?:In a real|In production|In the actual|In actual)/gi,
  /\/\/\s*(?:This is a |This is the |This should be a )?simplified\s+implementation/gi,
  /\/\/\s*For (?:now|simplicity|the time being),?\s*(?:we |just |simply )?(?:return|use|log)/gi,
  /\/\/\s*(?:Should|Would|Could) (?:call|use|fetch|load|query|connect to|integrate with)/gi,
  /\/\/\s*(?:Placeholder|Stub|Dummy|Hardcoded|Bypassed|Shortcut|Stubbed)/gi,
  // Chinese
  /\/\/\s*(?:在實際|在真實|實際實現|實際實作|簡化實現|簡化的實現|目前使用簡化)/g,
  /\/\/\s*(?:應該(?:從|調用|使用|連接|呼叫))/g,
  // Rust doc comments
  /\/\/\/?\s*(?:In a real|simplified|placeholder|stub|dummy)/gi,
  // Python/Ruby
  /#\s*(?:In a real|simplified|placeholder|stub|dummy|for now|HARDCODED)/gi,
];

export const R24_CHECKER: RuleChecker = {
  rule: getRedline('R24')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    if (/(?:test|spec|mock|fixture)/i.test(file)) return violations;
    const lines = source.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;
      for (const pattern of SELF_CONFESSING_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({
            ruleId: 'R24', ruleName: '自白註解', severity: 'error',
            file, line: i + 1, column: match.index + 1,
            message: `自白註解 (R24): 代碼自己承認是假實作 — ${match[0].substring(0, 50)}`,
            snippet: line.trim(),
            suggestion: '移除自白註解並實作真實邏輯，或明確標記 #[cfg(feature = "mock")]',
          });
          break;
        }
      }
    }
    return violations;
  },
};
