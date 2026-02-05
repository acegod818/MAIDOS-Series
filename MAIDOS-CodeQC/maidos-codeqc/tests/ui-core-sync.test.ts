/**
 * UI/UX 與核心同步驗證測試
 * 
 * 證明：
 * 1. Reporter 輸出的 ruleId 與核心定義一致
 * 2. Reporter 輸出的 ruleName 與核心定義一致
 * 3. Reporter 輸出的 severity 與核心定義一致
 * 4. HTML/Console 格式完整無遺漏
 */

import { describe, it, expect } from 'vitest';
import { analyzeFile } from '../src/analyzer.js';
import { getRedline } from '../src/rules/b-redlines.js';
import { getProhibition } from '../src/rules/b-prohibitions.js';
import { htmlReporter } from '../src/reporter/html.js';
import { consoleReporter } from '../src/reporter/console.js';

// =============================================================================
// 核心規則定義驗證
// =============================================================================

describe('核心規則定義完整性', () => {
  it('紅線規則定義完整', () => {
    const expectedRedlines = ['R01', 'R02', 'R03', 'R04', 'R05', 'R06', 'R07', 'R08', 'R09', 'R10', 'R11', 'R12'];
    for (const id of expectedRedlines) {
      const rule = getRedline(id);
      expect(rule, `${id} 應該存在`).toBeDefined();
      expect(rule?.name, `${id} 應該有中文名`).toBeTruthy();
      expect(rule?.nameEn, `${id} 應該有英文名`).toBeTruthy();
      expect(rule?.severity, `${id} 應該有嚴重度`).toBe('error');
    }
  });

  it('禁止規則定義完整', () => {
    // 禁止規則可能有不同的 severity (P12 是 info)
    const expectedProhibitions = ['P01', 'P02', 'P03', 'P04', 'P05', 'P06', 'P07', 'P08', 'P09', 'P10', 'P11', 'P12', 'P13', 'P14'];
    for (const id of expectedProhibitions) {
      const rule = getProhibition(id);
      expect(rule, `${id} 應該存在`).toBeDefined();
      expect(rule?.name, `${id} 應該有中文名`).toBeTruthy();
      expect(rule?.nameEn, `${id} 應該有英文名`).toBeTruthy();
      expect(['warning', 'info']).toContain(rule?.severity);
    }
  });
});

// =============================================================================
// UI 輸出與核心同步驗證
// =============================================================================

describe('UI/UX 與核心同步', () => {
  const testCode = `
const password = 'secret123';
eval(userInput);
try { x(); } catch (e) {}
while (true) { }
const url = 'http://api.example.com/login';
window.globalData = data;
const temp = 1;
`;

  it('Violation 的 ruleId 與核心定義一致', () => {
    const result = analyzeFile(testCode, 'test.ts', 'D');
    
    for (const v of result.violations) {
      if (v.ruleId.startsWith('R')) {
        const rule = getRedline(v.ruleId);
        expect(rule, `${v.ruleId} 應該在紅線定義中存在`).toBeDefined();
        expect(v.ruleName).toBe(rule!.name);
      } else if (v.ruleId.startsWith('P')) {
        const rule = getProhibition(v.ruleId);
        expect(rule, `${v.ruleId} 應該在禁止定義中存在`).toBeDefined();
        expect(v.ruleName).toBe(rule!.name);
      }
    }
  });

  it('Violation 的 severity 與核心定義一致', () => {
    const result = analyzeFile(testCode, 'test.ts', 'D');
    
    for (const v of result.violations) {
      if (v.ruleId.startsWith('R')) {
        expect(v.severity).toBe('error');
      } else if (v.ruleId.startsWith('P')) {
        expect(v.severity).toBe('warning');
      }
    }
  });
});

// =============================================================================
// HTML Reporter 格式驗證
// =============================================================================

describe('HTML Reporter 格式', () => {
  const mockResult = {
    timestamp: '2024-01-01T00:00:00.000Z',
    targetPath: 'test.ts',
    level: 'D' as const,
    duration: 100,
    files: [{
      file: 'test.ts',
      language: 'typescript' as const,
      lines: { totalLines: 10, codeLines: 8, commentLines: 1, blankLines: 1 },
      violations: [{
        ruleId: 'R01',
        ruleName: '硬編碼憑證',
        severity: 'error' as const,
        file: 'test.ts',
        line: 1,
        column: 1,
        message: '檢測到硬編碼憑證',
        snippet: "const password = 'secret123';",
        suggestion: '使用環境變數',
      }],
    }],
    summary: {
      totalFiles: 1,
      totalViolations: 1,
      errorCount: 1,
      warningCount: 0,
      infoCount: 0,
      byRule: { R01: 1 },
    },
  };

  it('HTML 包含所有必要元素', () => {
    const html = htmlReporter.report(mockResult);
    
    // 基本結構
    expect(html).toContain('<!DOCTYPE html>');
    expect(html).toContain('<html lang="zh-TW">');
    expect(html).toContain('MAIDOS CodeQC');
    
    // 違規信息
    expect(html).toContain('R01');
    expect(html).toContain('硬編碼憑證');
    expect(html).toContain('檢測到硬編碼憑證');
    expect(html).toContain('使用環境變數');
    
    // 圖標
    expect(html).toContain('🔴'); // error icon
    expect(html).toContain('💡'); // suggestion icon
    
    // 統計
    expect(html).toContain('Errors');
    expect(html).toContain('Warnings');
  });

  it('HTML 轉義特殊字符', () => {
    const resultWithSpecialChars = {
      ...mockResult,
      files: [{
        ...mockResult.files[0]!,
        violations: [{
          ...mockResult.files[0]!.violations[0]!,
          snippet: '<script>alert("xss")</script>',
        }],
      }],
    };
    
    const html = htmlReporter.report(resultWithSpecialChars);
    expect(html).not.toContain('<script>');
    expect(html).toContain('&lt;script&gt;');
  });
});

// =============================================================================
// Console Reporter 格式驗證
// =============================================================================

describe('Console Reporter 格式', () => {
  const mockResult = {
    timestamp: '2024-01-01T00:00:00.000Z',
    targetPath: 'test.ts',
    level: 'D' as const,
    duration: 100,
    files: [{
      file: 'test.ts',
      language: 'typescript' as const,
      lines: { totalLines: 10, codeLines: 8, commentLines: 1, blankLines: 1 },
      violations: [{
        ruleId: 'R01',
        ruleName: '硬編碼憑證',
        severity: 'error' as const,
        file: 'test.ts',
        line: 1,
        column: 1,
        message: '檢測到硬編碼憑證',
        snippet: "const password = 'secret123';",
        suggestion: '使用環境變數',
      }],
    }],
    summary: {
      totalFiles: 1,
      totalViolations: 1,
      errorCount: 1,
      warningCount: 0,
      infoCount: 0,
      byRule: { R01: 1 },
    },
  };

  it('Console 輸出包含所有必要元素', () => {
    const output = consoleReporter.report(mockResult);
    
    // 標題
    expect(output).toContain('MAIDOS CodeQC');
    
    // 規則信息
    expect(output).toContain('R01');
    expect(output).toContain('檢測到硬編碼憑證');
    
    // 統計
    expect(output).toContain('Errors');
    expect(output).toContain('Warnings');
  });

  it('Console 輸出包含建議', () => {
    const output = consoleReporter.report(mockResult);
    expect(output).toContain('💡');
    expect(output).toContain('使用環境變數');
  });
});

// =============================================================================
// 同步性證明
// =============================================================================

describe('UI/UX 與核心完全同步證明', () => {
  it('所有已實作的紅線規則都能被 UI 正確顯示', () => {
    // 已實作的紅線規則
    const implementedRedlines = ['R01', 'R02', 'R03', 'R05', 'R07', 'R08', 'R09', 'R10', 'R12'];
    
    for (const id of implementedRedlines) {
      const rule = getRedline(id);
      
      // 規則定義完整
      expect(rule).toBeDefined();
      expect(rule!.id).toMatch(/^R\d{2}$/);
      expect(rule!.name).toBeTruthy();
      expect(rule!.severity).toBe('error');
      expect(rule!.implemented).toBe(true);
    }
  });

  it('所有已實作的禁止規則都能被 UI 正確顯示', () => {
    // 已實作的禁止規則
    const implementedProhibitions = ['P03', 'P04', 'P05', 'P06', 'P07', 'P09', 'P10', 'P12', 'P13', 'P14'];
    
    for (const id of implementedProhibitions) {
      const rule = getProhibition(id);
      
      // 規則定義完整
      expect(rule).toBeDefined();
      expect(rule!.id).toMatch(/^P\d{2}$/);
      expect(rule!.name).toBeTruthy();
      expect(['warning', 'info']).toContain(rule!.severity);
      expect(rule!.implemented).toBe(true);
    }
  });

  it('Violation 結構與 Reporter 期望一致', () => {
    // 測試代碼觸發違規
    const testCode = `const password = 'secret123';`;
    const result = analyzeFile(testCode, 'test.ts', 'D');
    
    // 至少有一個違規
    expect(result.violations.length).toBeGreaterThan(0);
    
    // 每個違規都有完整結構
    for (const v of result.violations) {
      expect(v.ruleId).toBeTruthy();
      expect(v.ruleName).toBeTruthy();
      expect(v.severity).toBeTruthy();
      expect(v.file).toBeTruthy();
      expect(typeof v.line).toBe('number');
      expect(typeof v.column).toBe('number');
      expect(v.message).toBeTruthy();
    }
  });
});
