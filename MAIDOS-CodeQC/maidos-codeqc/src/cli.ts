#!/usr/bin/env node
/**
 * MAIDOS CodeQC CLI
 * 
 * 支援功能：
 * - 單檔案 / 批量資料夾掃描
 * - 軟配置選擇分析類型 (可複選)
 * - 多種輸出格式 (Console/JSON/HTML)
 */

import { readFileSync, writeFileSync, existsSync, statSync, readdirSync, type Dirent } from 'node:fs';
import { resolve, join, relative } from 'node:path';
import { analyze, getReporter, VERSION, CODEQC_VERSION } from './index.js';
import { pipelineCommand, serveCommand } from './commands.js';
import { SUPPORTED_EXTENSIONS } from './languages.js';
import { parse as parseYaml } from 'yaml';
import type { CheckLevel } from './types.js';

// =============================================================================
// Analysis Category Types (軟配置)
// =============================================================================

export type AnalysisCategory = 'security' | 'structure' | 'quality';

export interface AnalysisConfig {
  // 可複選的分析類型
  categories: Set<AnalysisCategory>;
  // 細粒度規則開關
  rules: {
    // 安全性分析 (Security)
    security: {
      credentials: boolean;      // R01 硬編碼憑證
      injection: boolean;        // R02 注入攻擊
      auditLogs: boolean;        // R03 審計日誌
      errorHandling: boolean;    // R05 錯誤處理
      securityDisable: boolean;  // R07 安全功能
      vulnerabilities: boolean;  // R08 已知漏洞
      resources: boolean;        // R09 資源限制
      plaintext: boolean;        // R10 明文傳輸
    };
    // 結構性分析 (Structure)
    structure: {
      longFunction: boolean;     // P05 超長函數
      deepNesting: boolean;      // P06 深層嵌套
      globalState: boolean;      // P07 全局狀態
      longParams: boolean;       // P10 過長參數
      copyPaste: boolean;        // P03 複製粘貼
    };
    // 代碼質量分析 (Quality)
    quality: {
      magicNumbers: boolean;     // P04 魔法數字
      naming: boolean;           // P09 命名規範
      todos: boolean;            // P13 待辦堆積
      comments: boolean;         // P12 註釋代碼
      dependencies: boolean;     // P14 依賴膨脹
    };
  };
}

// 預設配置：全部啟用
export const DEFAULT_CONFIG: AnalysisConfig = {
  categories: new Set(['security', 'structure', 'quality']),
  rules: {
    security: {
      credentials: true,
      injection: true,
      auditLogs: true,
      errorHandling: true,
      securityDisable: true,
      vulnerabilities: true,
      resources: true,
      plaintext: true,
    },
    structure: {
      longFunction: true,
      deepNesting: true,
      globalState: true,
      longParams: true,
      copyPaste: true,
    },
    quality: {
      magicNumbers: true,
      naming: true,
      todos: true,
      comments: true,
      dependencies: true,
    },
  },
};

// =============================================================================
// CLI Arguments Parser
// =============================================================================

interface CLIArgs {
  target: string;
  level: CheckLevel;
  reporter: 'console' | 'json' | 'html';
  output?: string;
  ci: boolean;
  help: boolean;
  version: boolean;
  // 軟配置：分析類型選擇 (可複選)
  categories: Set<AnalysisCategory>;
  configFile?: string;
}

type CliBase = Partial<Pick<CLIArgs, 'target' | 'level' | 'reporter' | 'output' | 'ci' | 'categories'>>;

function parseArgs(args: string[], base?: CliBase): CLIArgs {
  const result: CLIArgs = {
    target: '.',
    level: 'D',
    reporter: 'console',
    ci: false,
    help: false,
    version: false,
    categories: new Set(['security', 'structure', 'quality']), // 預設全選
  };

  // Apply base config first; CLI flags/positionals override it.
  if (base?.target) result.target = base.target;
  if (base?.level) result.level = base.level;
  if (base?.reporter) result.reporter = base.reporter;
  if (base?.output) result.output = base.output;
  if (typeof base?.ci === 'boolean') result.ci = base.ci;
  if (base?.categories && base.categories.size > 0) result.categories = new Set(base.categories);
  
  for (let i = 0; i < args.length; i++) {
    const arg = args[i]!;
    
    if (arg === '-h' || arg === '--help') {
      result.help = true;
    } else if (arg === '-v' || arg === '--version') {
      result.version = true;
    } else if (arg === '--ci') {
      result.ci = true;
    } else if (arg === '-l' || arg === '--level') {
      const level = args[++i]?.toUpperCase();
      if (level === 'B' || level === 'C' || level === 'D') {
        result.level = level;
      }
    } else if (arg === '-r' || arg === '--reporter') {
      const reporter = args[++i]?.toLowerCase();
      if (reporter === 'console' || reporter === 'json' || reporter === 'html') {
        result.reporter = reporter;
      }
    } else if (arg === '-o' || arg === '--output') {
      result.output = args[++i];
    } else if (arg === '-c' || arg === '--config') {
      result.configFile = args[++i];
    } else if (arg === '--category' || arg === '-C') {
      // 軟配置：可複選分析類型
      // 用法: --category security,structure 或 -C quality
      const cats = args[++i]?.toLowerCase().split(',') || [];
      result.categories = new Set();
      for (const cat of cats) {
        if (cat === 'security' || cat === 'sec' || cat === 's') {
          result.categories.add('security');
        } else if (cat === 'structure' || cat === 'struct' || cat === 't') {
          result.categories.add('structure');
        } else if (cat === 'quality' || cat === 'qual' || cat === 'q') {
          result.categories.add('quality');
        } else if (cat === 'all' || cat === 'a') {
          result.categories.add('security');
          result.categories.add('structure');
          result.categories.add('quality');
        }
      }
      // 如果沒有有效選擇，預設全選
      if (result.categories.size === 0) {
        result.categories = new Set(['security', 'structure', 'quality']);
      }
    } else if (arg === '--security' || arg === '--sec') {
      result.categories.add('security');
    } else if (arg === '--structure' || arg === '--struct') {
      result.categories.add('structure');
    } else if (arg === '--quality' || arg === '--qual') {
      result.categories.add('quality');
    } else if (arg === '--only-security') {
      result.categories = new Set(['security']);
    } else if (arg === '--only-structure') {
      result.categories = new Set(['structure']);
    } else if (arg === '--only-quality') {
      result.categories = new Set(['quality']);
    } else if (!arg.startsWith('-')) {
      result.target = arg;
    }
  }
  
  return result;
}

function findConfigFile(args: string[]): string | undefined {
  for (let i = 0; i < args.length; i++) {
    const a = args[i];
    if (a === '-c' || a === '--config') return args[i + 1];
  }
  return undefined;
}

function normalizeCategory(raw: string): AnalysisCategory | null {
  const v = raw.trim().toLowerCase();
  if (!v) return null;
  if (v === 'security' || v === 'sec' || v === 's') return 'security';
  if (v === 'structure' || v === 'struct' || v === 't') return 'structure';
  if (v === 'quality' || v === 'qual' || v === 'q') return 'quality';
  if (v === 'all' || v === 'a') return null; // handled by caller
  return null;
}

function normalizeLevel(raw: unknown): CheckLevel | null {
  const v = String(raw ?? '').trim().toUpperCase();
  return v === 'B' || v === 'C' || v === 'D' ? (v as CheckLevel) : null;
}

function normalizeReporter(raw: unknown): CLIArgs['reporter'] | null {
  const v = String(raw ?? '').trim().toLowerCase();
  return v === 'console' || v === 'json' || v === 'html' ? (v as CLIArgs['reporter']) : null;
}

async function loadCliConfig(configPath: string): Promise<CliBase> {
  const abs = resolve(configPath);
  if (!existsSync(abs)) {
    console.error(`Error: Config file not found: ${configPath}`);
    process.exit(1);
  }

  const content = readFileSync(abs, 'utf-8');
  const ext = '.' + (abs.split('.').pop() || '').toLowerCase();

  let parsed: unknown;
  try {
    if (ext === '.json') {
      parsed = JSON.parse(content);
    } else if (ext === '.yml' || ext === '.yaml') {
      parsed = parseYaml(content);
    } else {
      // Best-effort: try JSON first, then YAML.
      try { parsed = JSON.parse(content); }
      catch {
        parsed = parseYaml(content);
      }
    }
  } catch (err: unknown) {
    console.error(`Error: Failed to parse config file: ${configPath}`);
    console.error(err instanceof Error ? err.message : String(err));
    process.exit(1);
  }

  const base: CliBase = {};
  const cfg: Record<string, unknown> = (parsed && typeof parsed === 'object') ? (parsed as Record<string, unknown>) : {};

  if (typeof cfg.target === 'string' && cfg.target.trim()) base.target = cfg.target.trim();

  const level = normalizeLevel(cfg.level);
  if (level) base.level = level;

  const reporter = normalizeReporter(cfg.reporter);
  if (reporter) base.reporter = reporter;

  if (typeof cfg.output === 'string') base.output = cfg.output;
  if (typeof cfg.ci === 'boolean') base.ci = cfg.ci;

  // categories: "security,quality" | ["security","quality"]
  if (cfg.categories) {
    const rawCats: string[] = Array.isArray(cfg.categories)
      ? cfg.categories.map(String)
      : String(cfg.categories).split(',');

    const cats = new Set<AnalysisCategory>();
    for (const c of rawCats) {
      const normalized = normalizeCategory(String(c));
      if (normalized) cats.add(normalized);
    }
    // "all" means do not override defaults (or explicitly set to all).
    if (rawCats.some(c => ['all', 'a'].includes(String(c).trim().toLowerCase()))) {
      cats.add('security');
      cats.add('structure');
      cats.add('quality');
    }
    if (cats.size > 0) base.categories = cats;
  }

  return base;
}

// =============================================================================
// Constants
// =============================================================================

const RULE_COUNTS = {
  B_DISCIPLINE: 41,
  C_ACCEPTANCE: 50,
  D_COMBINED: 91,
} as const;

// =============================================================================
// Help & Version
// =============================================================================

const HELP = `
MAIDOS CodeQC v${VERSION} (Code-QC v${CODEQC_VERSION})

Usage:
  maidos-codeqc [options] [target]

Target:
  可以是單一檔案或資料夾，支援遞迴掃描
  Examples:
    maidos-codeqc ./src/app.ts          # 單一檔案
    maidos-codeqc ./src                  # 整個資料夾 (遞迴)
    maidos-codeqc .                      # 當前目錄

Options:
  -l, --level <B|C|D>       Check level (default: D)
                            B = 工作紀律 (${RULE_COUNTS.B_DISCIPLINE} rules)
                            C = 驗收標準 (~${RULE_COUNTS.C_ACCEPTANCE} rules)
                            D = B + C (~${RULE_COUNTS.D_COMBINED} rules)
  
  -C, --category <types>    分析類型 (可複選，逗號分隔)
                            security,sec,s  = 安全性分析 (R01-R12)
                            structure,struct,t = 結構性分析 (P03,P05-P07,P10)
                            quality,qual,q  = 代碼質量 (P04,P09,P12-P14)
                            all,a           = 全部 (預設)
                            
  --only-security           僅安全性分析
  --only-structure          僅結構性分析
  --only-quality            僅代碼質量分析
  
  -c, --config <file>       配置檔 (JSON/YAML)
  
  -r, --reporter <type>     輸出格式 (default: console)
                            console = 彩色終端
                            json    = JSON 格式
                            html    = HTML 報告
  
  -o, --output <file>       輸出檔案 (for json/html)
  
  --ci                      CI 模式 (有錯誤時 exit 1)
  
  -v, --version             顯示版本
  -h, --help                顯示說明

Supported Languages (43):
  Core:       TypeScript, JavaScript, Python, Rust, Go
  JVM:        Java, Kotlin, Scala, Groovy, Clojure
  .NET:       C#, F#, VB.NET
  Mobile:     Swift, Objective-C, Dart
  Systems:    C, C++, Zig, Nim
  Web:        PHP, Ruby
  Scripting:  Shell, PowerShell, Perl, Lua
  Data:       SQL, R, Julia
  Config:     YAML, JSON, TOML, XML
  Functional: Elixir, Haskell, OCaml, Erlang
  Enterprise: COBOL, ABAP, PL/SQL, Fortran, VBA, RPG

Examples:
  # 基本用法
  maidos-codeqc ./src                    # 掃描 ./src 資料夾
  maidos-codeqc ./app.py                 # 掃描單一檔案
  
  # 軟配置：選擇分析類型 (可複選)
  maidos-codeqc -C security ./src        # 僅安全性分析
  maidos-codeqc -C security,quality ./   # 安全性 + 代碼質量
  maidos-codeqc --only-structure ./src   # 僅結構性分析
  
  # 輸出格式
  maidos-codeqc -r html -o report.html   # HTML 報告
  maidos-codeqc -r json -o result.json   # JSON 輸出
  
  # CI/CD 整合
  maidos-codeqc --ci ./src               # CI 模式
`;

// =============================================================================
// File Discovery - 支援 43 種語言
// =============================================================================

const IGNORE_DIRS = [
  'node_modules', 'dist', 'build', '.git', 'vendor', '__pycache__', 'target',
  // .NET / MSBuild artifacts
  'bin', 'obj',
  // Common publish/output folders used across MAIDOS repos
  'dist_release',
  'publish', 'publish_new',
];

function discoverFiles(targetPath: string): string[] {
  const files: string[] = [];
  const absolutePath = resolve(targetPath);
  
  if (!existsSync(absolutePath)) {
    console.error(`Error: Path not found: ${targetPath}`);
    process.exit(1);
  }
  
  const stat = statSync(absolutePath);
  
  if (stat.isFile()) {
    const ext = '.' + absolutePath.split('.').pop()?.toLowerCase();
    if (SUPPORTED_EXTENSIONS.includes(ext)) {
      files.push(absolutePath);
    }
    return files;
  }
  
  function walk(dir: string) {
    // On some machines, scanning user/profile or drive roots will hit protected folders.
    // CodeQC should be stable: skip unreadable directories instead of crashing. The user can
    // always narrow the target path for stricter coverage.
    let entries: Dirent[];
    try {
      entries = readdirSync(dir, { withFileTypes: true });
    } catch (err: unknown) {
      const codeVal = (err && typeof err === 'object' && 'code' in err)
        ? String((err as Record<string, unknown>).code)
        : '';
      const code = codeVal ? ` (${codeVal})` : '';
      console.warn(`Warning: cannot read dir: ${dir}${code}`);
      return;
    }
    
    for (const entry of entries) {
      const fullPath = join(dir, entry.name);
      
      if (entry.isDirectory()) {
        if (!IGNORE_DIRS.includes(entry.name) && !entry.name.startsWith('.')) {
          walk(fullPath);
        }
      } else if (entry.isSymbolicLink()) {
        // Avoid unexpected symlink loops / junctions in large repos.
        continue;
      } else if (entry.isFile()) {
        const ext = '.' + entry.name.split('.').pop()?.toLowerCase();
        if (SUPPORTED_EXTENSIONS.includes(ext)) {
          files.push(fullPath);
        }
      }
    }
  }
  
  walk(absolutePath);
  return files;
}

// =============================================================================
// Helpers
// =============================================================================

function handleVersionFlag(): void {
  console.log(`MAIDOS CodeQC v${VERSION} (Code-QC v${CODEQC_VERSION})`);
  process.exit(0);
}

function handleHelpFlag(): void {
  console.log(HELP);
  process.exit(0);
}

export function loadFiles(targetPath: string): Array<{ path: string; content: string }> {
  const resolved = resolve(targetPath);
  const filePaths = discoverFiles(resolved);
  
  if (filePaths.length === 0) {
    console.log('No supported files found.');
    process.exit(0);
  }
  
  return filePaths.map(p => ({
    path: relative(process.cwd(), p),
    content: readFileSync(p, 'utf-8'),
  }));
}

async function outputReport(report: string, outputPath?: string): Promise<void> {
  if (outputPath) {
    writeFileSync(outputPath, report);
    console.log(`Report written to: ${outputPath}`);
  } else {
    console.log(report);
  }
}

// =============================================================================
// Category Filter - 根據軟配置篩選違規
// =============================================================================

const RULE_CATEGORIES: Record<string, AnalysisCategory> = {
  // Security (安全性)
  R01: 'security', R02: 'security', R03: 'security', R04: 'security',
  R05: 'security', R06: 'security', R07: 'security', R08: 'security',
  R09: 'security', R10: 'security', R11: 'security', R12: 'security',
  // Structure (結構性)
  P03: 'structure', P05: 'structure', P06: 'structure', P07: 'structure', P10: 'structure',
  // Quality (代碼質量)
  P04: 'quality', P09: 'quality', P12: 'quality', P13: 'quality', P14: 'quality',
  // 其他禁止規則歸類為 quality
  P01: 'quality', P02: 'quality', P08: 'quality', P11: 'quality',
};

function filterByCategory(result: ReturnType<typeof analyze>, categories: Set<AnalysisCategory>) {
  // 如果全選，不需要篩選
  if (categories.size === 3) return result;
  
  // 篩選違規
  for (const fileResult of result.files) {
    fileResult.violations = fileResult.violations.filter(v => {
      const cat = RULE_CATEGORIES[v.ruleId];
      return cat && categories.has(cat);
    });
  }
  
  // 重新計算統計
  let errorCount = 0;
  let warningCount = 0;
  let infoCount = 0;
  let totalViolations = 0;
  const byRule: Record<string, number> = {};
  
  for (const f of result.files) {
    for (const v of f.violations) {
      totalViolations++;
      byRule[v.ruleId] = (byRule[v.ruleId] || 0) + 1;
      if (v.severity === 'error') errorCount++;
      else if (v.severity === 'warning') warningCount++;
      else infoCount++;
    }
  }
  
  result.summary = {
    ...result.summary,
    totalViolations,
    errorCount,
    warningCount,
    infoCount,
    byRule: byRule as Record<string, number>,
  };
  
  return result;
}

// =============================================================================
// Main
// =============================================================================

async function main() {
  // v3.3 子命令路由: pipeline / serve / scan (預設)
  const rawArgs = process.argv.slice(2);
  const subcommand = rawArgs[0]?.toLowerCase();

  if (subcommand === 'pipeline') {
    return pipelineCommand(rawArgs.slice(1), loadFiles);
  }
  if (subcommand === 'serve') {
    return serveCommand(rawArgs.slice(1));
  }

  // 預設: scan (向下相容 v3.2)
  const cfgPath = findConfigFile(rawArgs);
  const base = cfgPath ? await loadCliConfig(cfgPath) : undefined;
  const args = parseArgs(rawArgs, base);
  
  if (args.version) handleVersionFlag();
  if (args.help) handleHelpFlag();
  
  // 顯示分析配置
  const categoryNames = {
    security: '安全性 (Security)',
    structure: '結構性 (Structure)',
    quality: '代碼質量 (Quality)',
  };
  const selectedCats = [...args.categories].map(c => categoryNames[c]).join(', ');
  console.log(`\n📊 分析類型: ${selectedCats}\n`);
  
  const files = loadFiles(args.target);
  let result = analyze({ files, level: args.level, targetPath: args.target });
  
  // 軟配置篩選
  result = filterByCategory(result, args.categories);
  
  // 將軟配置寫入結果供 UI 顯示
  result.categories = [...args.categories] as Array<'security' | 'structure' | 'quality'>;
  
  const reporter = getReporter(args.reporter);
  const report = await Promise.resolve(reporter.report(result));
  
  await outputReport(report, args.output);
  
  if (args.ci && result.summary.errorCount > 0) {
    process.exit(1);
  }
}

main().catch(err => {
  console.error('Error:', err.message);
  process.exit(1);
});
