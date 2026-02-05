/**
 * Type definitions for MAIDOS CodeQC Web Plugin
 */

export type Severity = 'error' | 'warning' | 'info';

export interface Violation {
  ruleId: string;
  ruleName: string;
  severity: Severity;
  file: string;
  line: number;
  column: number;
  message: string;
  snippet?: string;
  suggestion?: string;
}

export interface LanguagePlugin {
  name: string;
  extensions: string[];
  checkSource(source: string, file: string): Violation[];
}
