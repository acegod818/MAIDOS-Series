/**
 * MAIDOS CodeQC - Language Support Map
 *
 * Single source of truth for:
 * - Which file extensions are considered "supported" by the CLI discovery
 * - How analyzer detects language for stats + reporting
 *
 * This prevents "宣稱支援但 analyzer 靜默跳過" class of fraud/mismatch.
 */

import type { SupportedLanguage } from './language-types.js';

// All extensions MUST be lowercase and include the leading dot.
export const EXTENSION_TO_LANGUAGE: Record<string, SupportedLanguage> = {
  // Core (5)
  '.ts': 'typescript',
  '.tsx': 'typescript',
  '.js': 'javascript',
  '.jsx': 'javascript',
  '.mjs': 'javascript',
  '.cjs': 'javascript',
  '.py': 'python',
  '.rs': 'rust',
  '.go': 'go',

  // JVM (5)
  '.java': 'java',
  '.kt': 'kotlin',
  '.kts': 'kotlin',
  '.scala': 'scala',
  '.groovy': 'groovy',
  '.gvy': 'groovy',
  '.clj': 'clojure',
  '.cljs': 'clojure',
  '.cljc': 'clojure',

  // .NET (3)
  '.cs': 'csharp',
  '.fs': 'fsharp',
  '.fsx': 'fsharp',
  '.vb': 'vbnet',

  // Mobile (3)
  '.swift': 'swift',
  '.m': 'objc',
  '.mm': 'objc',
  '.dart': 'dart',

  // Systems (4)
  '.c': 'c',
  '.h': 'cpp',
  '.cpp': 'cpp',
  '.cxx': 'cpp',
  '.cc': 'cpp',
  '.hpp': 'cpp',
  '.hxx': 'cpp',
  '.zig': 'zig',
  '.nim': 'nim',

  // Web (2)
  '.php': 'php',
  '.rb': 'ruby',
  '.erb': 'ruby',

  // Scripting (4)
  '.sh': 'shell',
  '.bash': 'shell',
  '.zsh': 'shell',
  '.ps1': 'powershell',
  '.psm1': 'powershell',
  '.pl': 'perl',
  '.pm': 'perl',
  '.lua': 'lua',

  // Data (3)
  '.sql': 'sql',
  '.r': 'r',
  '.jl': 'julia',

  // Config (4)
  '.yaml': 'yaml',
  '.yml': 'yaml',
  '.json': 'json',
  '.toml': 'toml',
  '.xml': 'xml',

  // Functional (4)
  '.ex': 'elixir',
  '.exs': 'elixir',
  '.hs': 'haskell',
  '.lhs': 'haskell',
  '.ml': 'ocaml',
  '.mli': 'ocaml',
  '.erl': 'erlang',
  '.hrl': 'erlang',

  // Enterprise (6)
  '.cob': 'cobol',
  '.cbl': 'cobol',
  '.cpy': 'cobol',
  '.abap': 'abap',
  '.abs': 'abap',
  '.pls': 'plsql',
  '.plb': 'plsql',
  '.pks': 'plsql',
  '.pkb': 'plsql',
  '.f': 'fortran',
  '.f90': 'fortran',
  '.f95': 'fortran',
  '.for': 'fortran',
  '.bas': 'vba',
  '.cls': 'vba',
  '.frm': 'vba',
  '.vbs': 'vba',
  '.rpg': 'rpg',
  '.rpgle': 'rpg',
  '.sqlrpgle': 'rpg',
};

export const SUPPORTED_EXTENSIONS: string[] = Object.keys(EXTENSION_TO_LANGUAGE);

export function detectLanguageFromPath(file: string): SupportedLanguage | null {
  const ext = '.' + (file.split('.').pop() || '').toLowerCase();
  return EXTENSION_TO_LANGUAGE[ext] || null;
}

export function isSupportedPath(file: string): boolean {
  return detectLanguageFromPath(file) !== null;
}
