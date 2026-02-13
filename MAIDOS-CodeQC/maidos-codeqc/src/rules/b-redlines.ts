import type { RuleChecker, Violation } from '../types.js';

export type { Redline } from './b-redlines-defs.js';
export { REDLINES, getRedline, getImplementedRedlines, getUnimplementedRedlines, getRedlineStats } from './b-redlines-defs.js';
import { stripRustCfgTestBlocks } from './b-redlines-utils.js';

import { R01_CHECKER, R02_CHECKER, R03_CHECKER, R04_CHECKER, R05_CHECKER } from './b-redlines-r01-r05.js';
import { R06_CHECKER, R07_CHECKER, R08_CHECKER, R09_CHECKER, R10_CHECKER, R11_CHECKER, R12_CHECKER } from './b-redlines-r07-r12.js';
import { R13_CHECKER, R14_CHECKER, R15_CHECKER, R16_CHECKER, R17_CHECKER, R18_CHECKER } from './b-redlines-r13-r18.js';
import { R19_CHECKER, R20_CHECKER, R21_CHECKER, R22_CHECKER, R23_CHECKER, R24_CHECKER } from './b-redlines-r19-r24.js';
import { R25_CHECKER, R26_CHECKER, R27_CHECKER, R28_CHECKER } from './b-redlines-r25-r28.js';

export const REDLINE_CHECKERS: RuleChecker[] = [
  R01_CHECKER, R02_CHECKER, R03_CHECKER, R04_CHECKER, R05_CHECKER,
  R06_CHECKER, R07_CHECKER, R08_CHECKER, R09_CHECKER, R10_CHECKER, R11_CHECKER, R12_CHECKER,
  R13_CHECKER, R14_CHECKER, R15_CHECKER,
  R16_CHECKER, R17_CHECKER, R18_CHECKER,
  R19_CHECKER, R20_CHECKER, R21_CHECKER, R22_CHECKER, R23_CHECKER, R24_CHECKER,
  R25_CHECKER, R26_CHECKER, R27_CHECKER, R28_CHECKER,
];

/** 反詐欺專用檢查器 (R16-R28) — v3.5 完整版 */
export const ANTI_FRAUD_CHECKERS: RuleChecker[] = [
  R16_CHECKER, R17_CHECKER, R18_CHECKER,
  R19_CHECKER, R20_CHECKER, R21_CHECKER, R22_CHECKER, R23_CHECKER, R24_CHECKER,
  R25_CHECKER, R26_CHECKER, R27_CHECKER, R28_CHECKER,
];

/** 執行反詐欺掃描 — 生成 fraud.log 的資料 */
export function checkAntifraud(source: string, file: string): Violation[] {
  const ext = file.split('.').pop()?.toLowerCase() || '';
  const scanSource = ext === 'rs' ? stripRustCfgTestBlocks(source) : source;

  const violations: Violation[] = [];
  for (const checker of ANTI_FRAUD_CHECKERS) {
    if (checker.checkSource) violations.push(...checker.checkSource(scanSource, file));
  }
  return violations;
}

export function checkRedlines(source: string, file: string): Violation[] {
  const ext = file.split('.').pop()?.toLowerCase() || '';
  const scanSource = ext === 'rs' ? stripRustCfgTestBlocks(source) : source;

  const violations: Violation[] = [];
  for (const checker of REDLINE_CHECKERS) {
    if (checker.checkSource) violations.push(...checker.checkSource(scanSource, file));
  }
  return violations;
}

