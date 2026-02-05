/**
 * MAIDOS CodeQC - Core Analyzer
 */

import type {
  AnalysisResult,
  FileAnalysisResult,
  Violation,
  CheckLevel,
  
  SupportedLanguage,
  RuleId,
} from './types.js';
import { checkRules } from './rules/index.js';
import { detectLanguageFromPath, isSupportedPath } from './languages.js';

// =============================================================================
// Language Detection
// =============================================================================

export function detectLanguage(file: string): SupportedLanguage | null {
  return detectLanguageFromPath(file);
}

export function isSupported(file: string): boolean {
  return isSupportedPath(file);
}

// =============================================================================
// Line Statistics
// =============================================================================

interface LineStats {
  totalLines: number;
  codeLines: number;
  commentLines: number;
  blankLines: number;
}

type CommentBlock = { start: RegExp; end: RegExp };

const C_LIKE_LANGUAGES: ReadonlySet<SupportedLanguage> = new Set([
  // Core
  'typescript', 'javascript', 'rust', 'go',
  // JVM
  'java', 'kotlin', 'scala', 'groovy',
  // .NET
  'csharp',
  // Mobile
  'swift', 'objc', 'dart',
  // Systems
  'c', 'cpp', 'zig',
]);

function getCommentSyntax(language: SupportedLanguage): { line: RegExp[]; blocks: CommentBlock[] } {
  // Keep it line-based and conservative. This is used for stats only, not rule matching.
  if (C_LIKE_LANGUAGES.has(language)) {
    return {
      line: [/^\s*\/\//],
      blocks: [{ start: /^\s*\/\*/, end: /\*\/\s*$/ }],
    };
  }

  switch (language) {
    case 'python':
      // Treat triple-quoted blocks as comment-ish for counting.
      return {
        line: [/^\s*#/],
        blocks: [{ start: /^\s*['"]{3}/, end: /['"]{3}\s*$/ }],
      };

    case 'clojure':
      return { line: [/^\s*;/, /^\s*#_/], blocks: [] };

    case 'fsharp':
      return {
        line: [/^\s*\/\//],
        blocks: [{ start: /^\s*\(\*/, end: /\*\)\s*$/ }],
      };

    case 'vbnet':
      return { line: [/^\s*'/, /^\s*rem\b/i], blocks: [] };

    case 'nim':
      return {
        line: [/^\s*#/],
        blocks: [{ start: /^\s*#\[/, end: /\]#\s*$/ }],
      };

    // Web
    case 'php':
      return {
        line: [/^\s*\/\//, /^\s*#/],
        blocks: [{ start: /^\s*\/\*/, end: /\*\/\s*$/ }],
      };

    case 'ruby':
      return {
        line: [/^\s*#/],
        blocks: [{ start: /^\s*=begin\b/, end: /^\s*=end\b/ }],
      };

    // Scripting
    case 'shell':
      return { line: [/^\s*#/], blocks: [] };

    case 'powershell':
      return {
        line: [/^\s*#/],
        blocks: [{ start: /^\s*<#/, end: /#>\s*$/ }],
      };

    case 'perl':
      return { line: [/^\s*#/], blocks: [] };

    case 'lua':
      return {
        line: [/^\s*--/],
        blocks: [{ start: /^\s*--\[\[/, end: /\]\]\s*$/ }],
      };

    // Data
    case 'sql':
    case 'plsql':
      return {
        line: [/^\s*--/],
        blocks: [{ start: /^\s*\/\*/, end: /\*\/\s*$/ }],
      };

    case 'r':
      return { line: [/^\s*#/], blocks: [] };

    case 'julia':
      return {
        line: [/^\s*#/],
        blocks: [{ start: /^\s*#=/, end: /=#\s*$/ }],
      };

    // Config
    case 'yaml':
    case 'toml':
      return { line: [/^\s*#/], blocks: [] };

    case 'json':
      // Official JSON has no comments.
      return { line: [], blocks: [] };

    case 'xml':
      return { line: [], blocks: [{ start: /^\s*<!--/, end: /-->\s*$/ }] };

    // Functional
    case 'elixir':
      return { line: [/^\s*#/], blocks: [] };

    case 'haskell':
      return {
        line: [/^\s*--/],
        blocks: [{ start: /^\s*\{-/, end: /-\}\s*$/ }],
      };

    case 'ocaml':
      return { line: [], blocks: [{ start: /^\s*\(\*/, end: /\*\)\s*$/ }] };

    case 'erlang':
      return { line: [/^\s*%/], blocks: [] };

    // Enterprise
    case 'cobol':
      return { line: [/^\s*\*>/, /^\s*\*/], blocks: [] };

    case 'abap':
      return { line: [/^\s*\*/, /^\s*"/], blocks: [] };

    case 'fortran':
      return { line: [/^\s*!/, /^[cC*]/], blocks: [] };

    case 'vba':
      return { line: [/^\s*'/, /^\s*rem\b/i], blocks: [] };

    case 'rpg':
      return { line: [/^\s*\/\//, /^\s*\*/], blocks: [] };

    default:
      // Should be unreachable (SupportedLanguage is a closed union), but keep a safe fallback.
      return { line: [], blocks: [] };
  }
}

function countLines(source: string, language: SupportedLanguage): LineStats {
  const lines = source.split('\n');
  let codeLines = 0;
  let commentLines = 0;
  let blankLines = 0;
  let inBlockComment: CommentBlock | null = null;
  const syntax = getCommentSyntax(language);
  
  for (const line of lines) {
    const trimmed = line.trim();
    
    if (trimmed === '') {
      blankLines++;
      continue;
    }
    
    if (inBlockComment) {
      commentLines++;
      if (inBlockComment.end.test(trimmed)) {
        inBlockComment = null;
      }
      continue;
    }
    
    let matched = false;
    
    for (const block of syntax.blocks) {
      if (block.start.test(trimmed)) {
        commentLines++;
        matched = true;
        if (!block.end.test(trimmed)) {
          inBlockComment = block;
        }
        break;
      }
    }

    if (matched) continue;

    for (const lc of syntax.line) {
      if (lc.test(trimmed)) {
        commentLines++;
        matched = true;
        break;
      }
    }

    if (matched) continue;
    
    codeLines++;
  }
  
  return {
    totalLines: lines.length,
    codeLines,
    commentLines,
    blankLines,
  };
}

// =============================================================================
// File Analyzer
// =============================================================================

export function analyzeFile(
  source: string,
  file: string,
  level: CheckLevel
): FileAnalysisResult {
  const startTime = performance.now();
  
  const language = detectLanguage(file);
  if (!language) {
    return {
      file,
      language: 'typescript', // fallback
      violations: [],
      duration: 0,
      stats: { totalLines: 0, codeLines: 0, commentLines: 0, blankLines: 0 },
    };
  }
  
  const violations = checkRules(source, file, level);
  const stats = countLines(source, language);
  
  const endTime = performance.now();
  
  return {
    file,
    language,
    violations,
    duration: Math.round(endTime - startTime),
    stats,
  };
}

// =============================================================================
// Batch Analyzer
// =============================================================================

export interface AnalyzeOptions {
  files: Array<{ path: string; content: string }>;
  level: CheckLevel;
  targetPath: string;
}

export function analyze(options: AnalyzeOptions): AnalysisResult {
  const startTime = performance.now();
  const { files, level, targetPath } = options;
  
  const fileResults: FileAnalysisResult[] = [];
  const byRule: Record<RuleId, number> = {} as Record<RuleId, number>;
  
  let errorCount = 0;
  let warningCount = 0;
  let infoCount = 0;
  
  for (const { path, content } of files) {
    const result = analyzeFile(content, path, level);
    fileResults.push(result);
    
    for (const v of result.violations) {
      byRule[v.ruleId] = (byRule[v.ruleId] || 0) + 1;
      
      switch (v.severity) {
        case 'error': errorCount++; break;
        case 'warning': warningCount++; break;
        case 'info': infoCount++; break;
      }
    }
  }
  
  const endTime = performance.now();
  
  return {
    timestamp: new Date().toISOString(),
    targetPath,
    level,
    files: fileResults,
    summary: {
      totalFiles: files.length,
      totalViolations: errorCount + warningCount + infoCount,
      errorCount,
      warningCount,
      infoCount,
      byRule,
    },
    duration: Math.round(endTime - startTime),
  };
}

// =============================================================================
// Quick Check (Single File)
// =============================================================================

export function quickCheck(source: string, file: string): Violation[] {
  return checkRules(source, file, 'D');
}
