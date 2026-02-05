/**
 * Console Reporter
 * 輸出彩色終端報告
 */

import type { AnalysisResult, Reporter, Violation, Severity } from '../types.js';

// ANSI 顏色碼（不使用 chalk 以減少依賴）
const colors = {
  reset: '\x1b[0m',
  bold: '\x1b[1m',
  dim: '\x1b[2m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
  white: '\x1b[37m',
  bgRed: '\x1b[41m',
  bgGreen: '\x1b[42m',
  bgYellow: '\x1b[43m',
};

function colorize(text: string, ...codes: string[]): string {
  return `${codes.join('')}${text}${colors.reset}`;
}

function severityIcon(severity: Severity): string {
  switch (severity) {
    case 'error': return colorize('🔴', colors.red);
    case 'warning': return colorize('🟡', colors.yellow);
    case 'info': return colorize('🔵', colors.blue);
  }
}

function severityColor(severity: Severity): string {
  switch (severity) {
    case 'error': return colors.red;
    case 'warning': return colors.yellow;
    case 'info': return colors.blue;
  }
}

function formatViolation(v: Violation): string {
  const icon = severityIcon(v.severity);
  const ruleId = colorize(v.ruleId, colors.bold, severityColor(v.severity));
  const location = colorize(`${v.file}:${v.line}:${v.column}`, colors.dim);
  
  let output = `${icon} ${ruleId} ${location}\n`;
  output += `   ${v.message}\n`;
  
  if (v.snippet) {
    output += colorize(`   > ${v.snippet}\n`, colors.dim);
  }
  
  if (v.suggestion) {
    output += colorize(`   💡 ${v.suggestion}\n`, colors.cyan);
  }
  
  return output;
}

export const consoleReporter: Reporter = {
  name: 'console',
  
  report(result: AnalysisResult): string {
    const lines: string[] = [];
    
    // Header
    lines.push('');
    lines.push(colorize('═══════════════════════════════════════════════════════════════', colors.dim));
    lines.push(colorize('  MAIDOS CodeQC v2.4 Analysis Report', colors.bold, colors.cyan));
    lines.push(colorize('═══════════════════════════════════════════════════════════════', colors.dim));
    lines.push('');
    
    // Meta info
    lines.push(`📂 Target: ${colorize(result.targetPath, colors.bold)}`);
    lines.push(`📊 Level: ${colorize(result.level, colors.bold)}`);
    lines.push(`⏱️  Duration: ${colorize(`${result.duration}ms`, colors.dim)}`);
    lines.push(`📁 Files: ${colorize(String(result.summary.totalFiles), colors.bold)}`);
    lines.push('');
    
    // Violations by file
    if (result.summary.totalViolations > 0) {
      lines.push(colorize('─── Violations ───', colors.dim));
      lines.push('');
      
      for (const fileResult of result.files) {
        if (fileResult.violations.length === 0) continue;
        
        lines.push(colorize(`📄 ${fileResult.file}`, colors.bold));
        lines.push('');
        
        for (const violation of fileResult.violations) {
          lines.push(formatViolation(violation));
        }
      }
    } else {
      lines.push(colorize('✅ No violations found!', colors.green, colors.bold));
      lines.push('');
    }
    
    // Summary
    lines.push(colorize('─── Summary ───', colors.dim));
    lines.push('');
    
    const { errorCount, warningCount, infoCount } = result.summary;
    
    lines.push(`${severityIcon('error')} Errors:   ${colorize(String(errorCount), errorCount > 0 ? colors.red : colors.green)}`);
    lines.push(`${severityIcon('warning')} Warnings: ${colorize(String(warningCount), warningCount > 0 ? colors.yellow : colors.green)}`);
    lines.push(`${severityIcon('info')} Info:     ${colorize(String(infoCount), colors.blue)}`);
    lines.push('');
    
    // Gate status (if available)
    if (result.gates) {
      lines.push(colorize('─── Gate Status ───', colors.dim));
      lines.push('');
      
      const gateStatus = (passed: boolean) => passed 
        ? colorize('✅ PASS', colors.green, colors.bold)
        : colorize('❌ FAIL', colors.red, colors.bold);
      
      lines.push(`Gate-In:     ${gateStatus(result.gates.gateIn.passed)}`);
      lines.push(`Gate-Mid:    ${gateStatus(result.gates.gateMid.passed)}`);
      lines.push(`Gate-Out:    ${gateStatus(result.gates.gateOut.passed)}`);
      lines.push(`Gate-Accept: ${gateStatus(result.gates.gateAccept.passed)}`);
      lines.push('');
    }
    
    // Dual-axis score (if available)
    if (result.score) {
      lines.push(colorize('─── Dual-Axis Score ───', colors.dim));
      lines.push('');
      
      const gradeColor = (grade: string) => {
        switch (grade) {
          case 'A': return colors.green;
          case 'B': return colors.blue;
          case 'C': return colors.yellow;
          case 'D': return colors.red;
          default: return colors.white;
        }
      };
      
      lines.push(`X-Axis (Compliance): ${colorize(`${result.score.x.total}%`, colors.bold)}`);
      lines.push(`Y-Axis (Outcome):    ${colorize(`${result.score.y.total}%`, colors.bold)}`);
      lines.push(`Grade: ${colorize(result.score.grade, colors.bold, gradeColor(result.score.grade))}`);
      lines.push('');
    }
    
    // Final verdict
    const passed = result.summary.errorCount === 0;
    lines.push(colorize('═══════════════════════════════════════════════════════════════', colors.dim));
    if (passed) {
      lines.push(colorize('  ✅ Gate-Out: PASS', colors.green, colors.bold));
    } else {
      lines.push(colorize('  ❌ Gate-Out: FAIL', colors.red, colors.bold));
    }
    lines.push(colorize('═══════════════════════════════════════════════════════════════', colors.dim));
    lines.push('');
    
    return lines.join('\n');
  },
};

export default consoleReporter;
