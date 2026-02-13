/**
 * Code-QC v3.5 — R25-R28 最終補漏規則
 *
 * 來源: MAIDOS-Series 深掃，發現 v3.4 R13-R24 仍有 4 類漏網之魚
 * 合計殘留: R25=58, R26=51, R27=31, R28=10 (150 實例)
 *
 * R25 日誌灌水   — 函數只有 log + trivial return，看起來有做事但沒做
 * R26 幽靈異步   — async fn 內部沒有任何 .await，假裝是異步操作
 * R27 錯誤洗白   — unwrap_or_default() 在關鍵路徑吞掉錯誤
 * R28 複製軍團   — 跨檔案完全相同的函數體，結構膨脹灌水
 *
 * 分層防禦模型:
 *   第一層 空殼   → R13/R15/R16/R20 (最笨的)
 *   第二層 假數據 → R17/R19/R22 (有值但假的)
 *   第三層 假邏輯 → R23/R25/R26 (看起來有做事) ← R25/R26 新增
 *   第四層 假整合 → R14/R18/R21/R27 (宣稱接外部) ← R27 新增
 *   第五層 假完整 → R24/R28 (結構膨脹)          ← R28 新增
 */

import type { RuleChecker, Violation } from '../types.js';
import { getRedline } from './b-redlines-defs.js';

// =============================================================================
// R25: 日誌灌水 (Log Stuffing)
// =============================================================================
//
// 手法: fn backup_drivers(&self) -> Result<()> {
//           log::info!("開始備份驅動程式");
//           Ok(())
//       }
//
// 為什麼危險: 看 log 以為功能正常運作，實際什麼都沒做
// 本次掃描命中: 58 個函數
// 誤報排除: new/default/fmt/drop/init (允許 init 只做 log)
//           → 改為: init 也不允許（上次審計 init 全是假的）
//           test 檔案跳過
//
// 檢測邏輯:
//   1. 找 pub fn / pub async fn
//   2. 解析函數體
//   3. 移除所有 log/tracing/info/debug/warn/error 行
//   4. 移除所有註解行
//   5. 剩餘只有 trivial return → 命中
// =============================================================================

// Trivial return values that don't constitute real work
const TRIVIAL_RETURNS_RS = new Set([
  'Ok(())', 'Ok(true)', 'Ok(false)', 'Ok(None)', 'Ok(vec![])',
  'Ok(String::new())', 'Ok(Default::default())', 'Ok(HashMap::new())',
  'Ok(Vec::new())', 'Ok(BTreeMap::new())', 'Ok(HashSet::new())',
  'true', 'false', 'None', '()', 'Default::default()',
  'String::new()', 'Vec::new()', 'HashMap::new()', '0', '0.0', '""',
]);

const LOG_LINE_RS = /^\s*(?:log|tracing|println|eprintln|debug|info|warn|error|trace)(?:::)?\w*!?\s*\(/;
const LOG_LINE_CS = /^\s*(?:Console\.Write|Debug\.Log|Logger\.\w+|_?logger\.\w+|Log\.\w+)\s*\(/;
const LOG_LINE_CPP = /^\s*(?:std::cout|std::cerr|printf|fprintf|spdlog|LOG_|SPDLOG_)\s*[(<]/;

// Whitelist: function names where log-only is acceptable
const LOG_ONLY_WHITELIST = new Set(['drop', 'fmt', 'display', 'debug', 'new', 'default']);

export const R25_CHECKER: RuleChecker = {
  rule: getRedline('R25')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    if (/(?:test|spec|mock|fixture|__test__)/i.test(file)) return violations;

    const ext = file.split('.').pop()?.toLowerCase() || '';
    const logPattern = ext === 'cs' ? LOG_LINE_CS : ext === 'cpp' || ext === 'h' ? LOG_LINE_CPP : LOG_LINE_RS;

    // Rust pattern: pub [async] fn name(...) ... { body }
    // We need to handle nested braces, so we do a simplified scan
    const fnPattern = ext === 'rs'
      ? /(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\([^)]*\)[^{]*\{/g
      : ext === 'cs'
        ? /(?:public|internal|private|protected)\s+(?:static\s+)?(?:async\s+)?[\w<>\[\]]+\s+(\w+)\s*\([^)]*\)\s*\{/g
        : /(?:void|bool|int|string|auto)\s+(\w+)\s*\([^)]*\)\s*\{/g;

    let match;
    while ((match = fnPattern.exec(source)) !== null) {
      const fnName = match[1]!;
      if (LOG_ONLY_WHITELIST.has(fnName.toLowerCase())) continue;

      // Extract function body (simplified: find matching closing brace)
      const bodyStart = match.index + match[0].length;
      let depth = 1;
      let pos = bodyStart;
      while (pos < source.length && depth > 0) {
        if (source[pos] === '{') depth++;
        else if (source[pos] === '}') depth--;
        pos++;
      }
      if (depth !== 0) continue;
      const body = source.substring(bodyStart, pos - 1);

      // Skip large function bodies (>500 chars = likely has real logic)
      if (body.length > 500) continue;

      const lines = body.split('\n')
        .map(l => l.trim())
        .filter(l => l && !l.startsWith('//') && !l.startsWith('/*') && !l.startsWith('*'));

      if (lines.length === 0) continue; // R16 handles this

      // Remove log lines
      const afterLog = lines.filter(l => !logPattern.test(l));

      // Check if remaining lines are all trivial returns
      const meaningful = afterLog.filter(l => {
        const trimmed = l.replace(/;$/, '').trim();
        return !TRIVIAL_RETURNS_RS.has(trimmed) && trimmed !== '';
      });

      if (meaningful.length === 0 && lines.length >= 1) {
        const lineNum = (source.substring(0, match.index).match(/\n/g) || []).length + 1;
        violations.push({
          ruleId: 'R25', ruleName: '日誌灌水', severity: 'error',
          file, line: lineNum, column: 1,
          message: `日誌灌水 (R25): fn ${fnName}() 只有 ${lines.length} 行 log + trivial return，無實際業務邏輯`,
          snippet: `fn ${fnName}() { ${lines[0]?.substring(0, 40) ?? ''}... }`,
          suggestion: '函數體必須包含真實業務邏輯（I/O、計算、狀態變更），不能只有日誌',
        });
      }
    }
    return violations;
  },
};

// =============================================================================
// R26: 幽靈異步 (Phantom Async)
// =============================================================================
//
// 手法: pub async fn download_driver(&self, ...) -> Result<()> {
//           let path = format!("...");
//           // 純同步操作，沒有任何 .await
//           Ok(())
//       }
//
// 為什麼危險: async 暗示會做異步 I/O (網路/檔案)，沒有 .await
//             代表根本沒去做那些 I/O 操作
// 本次掃描命中: 51 個函數
// 誤報排除: 
//   - body 裡有 .await 在嵌套 block → 我們的 regex 只看頂層
//   - body 很大 (>1000 chars) → 可能有 .await 在深層
//   - test 檔案
//   - trait default impl (有些 trait 要求 async 但某些 impl 不需要)
// =============================================================================

export const R26_CHECKER: RuleChecker = {
  rule: getRedline('R26')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    if (/(?:test|spec|mock|fixture)/i.test(file)) return violations;

    const ext = file.split('.').pop()?.toLowerCase() || '';
    
    // Only check Rust and C# (languages with async/await)
    if (ext !== 'rs' && ext !== 'cs') return violations;

    const asyncFnPattern = ext === 'rs'
      ? /pub\s+async\s+fn\s+(\w+)\s*\([^)]*\)[^{]*\{/g
      : /(?:public|internal)\s+(?:static\s+)?async\s+[\w<>\[\]]+\s+(\w+)\s*\([^)]*\)\s*\{/g;

    let match;
    while ((match = asyncFnPattern.exec(source)) !== null) {
      const fnName = match[1]!;
      
      // Extract body
      const bodyStart = match.index + match[0].length;
      let depth = 1;
      let pos = bodyStart;
      while (pos < source.length && depth > 0) {
        if (source[pos] === '{') depth++;
        else if (source[pos] === '}') depth--;
        pos++;
      }
      if (depth !== 0) continue;
      const body = source.substring(bodyStart, pos - 1);
      
      // Skip very large bodies (might have .await in nested blocks our scan misses)
      if (body.length > 2000) continue;

      // Check for .await (Rust) or await keyword (C#)
      const hasAwait = ext === 'rs'
        ? /\.await\b/.test(body)
        : /\bawait\b/.test(body);
      
      if (!hasAwait) {
        const lineNum = (source.substring(0, match.index).match(/\n/g) || []).length + 1;
        violations.push({
          ruleId: 'R26', ruleName: '幽靈異步', severity: 'error',
          file, line: lineNum, column: 1,
          message: `幽靈異步 (R26): async fn ${fnName}() 內部沒有任何 .await 調用`,
          snippet: `async fn ${fnName}() { ... no .await ... }`,
          suggestion: 'async 函數必須有 .await 調用（網路/檔案/channel），否則移除 async',
        });
      }
    }
    return violations;
  },
};

// =============================================================================
// R27: 錯誤洗白 (Error Laundering)
// =============================================================================
//
// 手法: let private_key = hex::decode(&key).unwrap_or_default();
//       // 解碼失敗 → 得到空 Vec → 簽名用空 key → 安全漏洞
//
//       let checksum = compute_hash(file).unwrap_or_default();
//       // 計算失敗 → 空字串 → 校驗比對永遠不通過或永遠通過
//
// 為什麼危險: 用 unwrap_or_default/unwrap_or(0)/unwrap_or(false)
//             把錯誤靜默轉成空值/零值，下游代碼不知道出錯了
//             在安全相關操作中尤其致命
// 本次掃描命中: 31 個在高風險函數中
// 誤報排除:
//   - test 檔案
//   - 只檢查高風險函數（verify/auth/sign/encrypt/download/install/delete/
//     backup/restore/payment/transfer/checksum/hash/validate/parse/load/
//     save/query/connect/send/fetch/wallet/key/secret/credential）
//   - unwrap_or_default() 在低風險上下文（如 token count、color）不觸發
// =============================================================================

const HIGH_RISK_CONTEXTS = /(?:verify|auth|sign|encrypt|decrypt|download|install|delete|backup|restore|payment|transfer|checksum|hash|validate|wallet|key|secret|credential|private|password|token_refresh|certificate)/i;

const ERROR_LAUNDER_PATTERNS = [
  /\.unwrap_or_default\(\)/g,
  /\.unwrap_or\(\s*(?:vec!\[\]|String::new\(\)|Vec::new\(\)|HashMap::new\(\)|false|0|"")\s*\)/g,
];

export const R27_CHECKER: RuleChecker = {
  rule: getRedline('R27')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    if (/(?:test|spec|mock|fixture)/i.test(file)) return violations;

    const ext = file.split('.').pop()?.toLowerCase() || '';
    if (ext !== 'rs') return violations; // unwrap_or is Rust-specific

    const lines = source.split('\n');
    let currentFn = '';

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]!;

      // Track current function name
      const fnMatch = /(?:pub\s+)?(?:async\s+)?fn\s+(\w+)/.exec(line);
      if (fnMatch) currentFn = fnMatch[1]!;

      // Only flag in high-risk function contexts
      if (!HIGH_RISK_CONTEXTS.test(currentFn) && !HIGH_RISK_CONTEXTS.test(line)) continue;

      for (const pattern of ERROR_LAUNDER_PATTERNS) {
        pattern.lastIndex = 0;
        const match = pattern.exec(line);
        if (match) {
          violations.push({
            ruleId: 'R27', ruleName: '錯誤洗白', severity: 'error',
            file, line: i + 1, column: match.index + 1,
            message: `錯誤洗白 (R27): ${match[0]} 在安全相關函數 ${currentFn}() 中靜默吞掉錯誤`,
            snippet: line.trim().substring(0, 80),
            suggestion: '安全相關操作必須用 ? 傳播錯誤或明確處理失敗，不能 unwrap_or_default',
          });
          break;
        }
      }
    }
    return violations;
  },
};

// =============================================================================
// R28: 複製軍團 (Clone Army)
// =============================================================================
//
// 手法: 10 個 social 平台的 spawn_http_server() 完全相同
//       5 個 LLM provider 的 build_request() 完全相同
//       看起來功能豐富（10 個平台全支援！）但其實是複製貼上
//
// 為什麼危險:
//   1. 應該不同的邏輯被複製成相同 → 實際沒有針對各平台做差異化
//   2. 結構膨脹 → 給人「項目很大很完整」的假象
//   3. 維護地獄 → 改一處要改 N 處，很快就 diverge 成 bug
//
// 本次掃描命中: 10 組 pattern，跨 3+ 檔案
//
// 檢測方式: 單檔 checker 無法做跨檔案比對
//   → 這個 checker 使用啟發式：偵測函數體中包含「可疑的完全
//     通用代碼」特徵（如 TcpListener::bind("127.0.0.1:0")
//     出現在非 test/非 util 檔案中）
//   → 完整的跨檔案查重由 batch 工具 (duplicate-body-scanner) 執行
//
// 本 checker 標記的是「抄作業痕跡」：
//   - 非 test 代碼中出現 TcpListener::bind("127.0.0.1:0") (test server)
//   - 非 util 代碼中整段複製 HTTP client 樣板
// =============================================================================

const CLONE_ARMY_PATTERNS = [
  // Test server in production code
  /TcpListener::bind\(\s*"127\.0\.0\.1:0"\s*\)/g,
  // Hardcoded test responses in production
  /spawn_http_server\s*\(\s*"(?:200 OK|404|500)/g,
  // Copy-paste HTTP client boilerplate with hardcoded content-type
  /\.header\(\s*"Content-Type"\s*,\s*"application\/json"\s*\)[\s\S]{0,100}\.header\(\s*"Authorization"/g,
];

export const R28_CHECKER: RuleChecker = {
  rule: getRedline('R28')!,
  checkSource(source: string, file: string): Violation[] {
    const violations: Violation[] = [];
    if (/(?:test|spec|mock|fixture)/i.test(file)) return violations;

    for (const pattern of CLONE_ARMY_PATTERNS) {
      pattern.lastIndex = 0;
      let match;
      while ((match = pattern.exec(source)) !== null) {
        const before = source.substring(0, match.index);
        const lineNum = (before.match(/\n/g) || []).length + 1;
        violations.push({
          ruleId: 'R28', ruleName: '複製軍團', severity: 'error',
          file, line: lineNum, column: 1,
          message: `複製軍團 (R28): 非測試代碼中出現測試/樣板代碼 — ${match[0].substring(0, 50)}`,
          snippet: match[0].substring(0, 80),
          suggestion: '抽取為共用 util 函數，各模組調用共用代碼而非複製',
        });
      }
    }
    return violations;
  },
};
