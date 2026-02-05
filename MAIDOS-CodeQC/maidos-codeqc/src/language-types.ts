/**
 * CodeQC language ids (single source of truth for the union type).
 *
 * Keep this file <500 lines (self-proof gate).
 */

/** 支援的語言（43）。避免「CLI 收進來但 analyzer 靜默跳過」的規格詐欺。 */
export type SupportedLanguage =
  | 'typescript' | 'javascript' | 'python' | 'rust' | 'go'
  | 'java' | 'kotlin' | 'scala' | 'groovy' | 'clojure'
  | 'csharp' | 'fsharp' | 'vbnet'
  | 'swift' | 'objc' | 'dart'
  | 'c' | 'cpp' | 'zig' | 'nim'
  | 'php' | 'ruby'
  | 'shell' | 'powershell' | 'perl' | 'lua'
  | 'sql' | 'r' | 'julia'
  | 'yaml' | 'json' | 'toml' | 'xml'
  | 'elixir' | 'haskell' | 'ocaml' | 'erlang'
  | 'cobol' | 'abap' | 'plsql' | 'fortran' | 'vba' | 'rpg';

