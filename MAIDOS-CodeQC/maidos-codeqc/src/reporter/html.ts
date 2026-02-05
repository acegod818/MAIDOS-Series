/**
 * HTML Reporter
 * 輸出可視化 HTML 報告
 */

import type { AnalysisResult, Reporter, Violation, Severity, AnalysisCategory } from '../types.js';

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

function renderCategories(categories: AnalysisCategory[]): string {
  const names: Record<AnalysisCategory, string> = {
    security: '🔒 安全性',
    structure: '🏗️ 結構性',
    quality: '✨ 代碼質量',
  };
  return categories.map(c => names[c]).join(' + ');
}

function severityClass(severity: Severity): string {
  switch (severity) {
    case 'error': return 'severity-error';
    case 'warning': return 'severity-warning';
    case 'info': return 'severity-info';
  }
}

function severityIcon(severity: Severity): string {
  switch (severity) {
    case 'error': return '🔴';
    case 'warning': return '🟡';
    case 'info': return '🔵';
  }
}

function renderViolation(v: Violation): string {
  return `
    <div class="violation ${severityClass(v.severity)}">
      <div class="violation-header">
        <span class="severity-icon">${severityIcon(v.severity)}</span>
        <span class="rule-id">${escapeHtml(v.ruleId)}</span>
        <span class="rule-name">${escapeHtml(v.ruleName)}</span>
        <span class="location">${escapeHtml(v.file)}:${v.line}:${v.column}</span>
      </div>
      <div class="violation-message">${escapeHtml(v.message)}</div>
      ${v.snippet ? `<pre class="snippet">${escapeHtml(v.snippet)}</pre>` : ''}
      ${v.suggestion ? `<div class="suggestion">💡 ${escapeHtml(v.suggestion)}</div>` : ''}
    </div>
  `;
}

export const htmlReporter: Reporter = {
  name: 'html',
  
  report(result: AnalysisResult): string {
    const { summary, score, gates } = result;
    const passed = summary.errorCount === 0;
    
    return `<!DOCTYPE html>
<html lang="zh-TW">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>MAIDOS CodeQC Report</title>
  <style>
    :root {
      --color-error: #ef4444;
      --color-warning: #f59e0b;
      --color-info: #3b82f6;
      --color-success: #22c55e;
      --color-bg: #0f172a;
      --color-card: #1e293b;
      --color-text: #f8fafc;
      --color-dim: #94a3b8;
    }
    
    * { box-sizing: border-box; margin: 0; padding: 0; }
    
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      background: var(--color-bg);
      color: var(--color-text);
      line-height: 1.6;
      padding: 2rem;
    }
    
    .container { max-width: 1200px; margin: 0 auto; }
    
    h1 {
      font-size: 2rem;
      margin-bottom: 0.5rem;
      background: linear-gradient(135deg, #06b6d4, #3b82f6);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
    }
    
    .meta {
      color: var(--color-dim);
      margin-bottom: 2rem;
    }
    
    .summary-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 1rem;
      margin-bottom: 2rem;
    }
    
    .card {
      background: var(--color-card);
      border-radius: 12px;
      padding: 1.5rem;
    }
    
    .card-title {
      color: var(--color-dim);
      font-size: 0.875rem;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      margin-bottom: 0.5rem;
    }
    
    .card-value {
      font-size: 2rem;
      font-weight: bold;
    }
    
    .card-value.error { color: var(--color-error); }
    .card-value.warning { color: var(--color-warning); }
    .card-value.info { color: var(--color-info); }
    .card-value.success { color: var(--color-success); }
    
    .analysis-categories {
      font-size: 1.1rem;
      color: var(--color-text);
      margin-top: 0.5rem;
    }
    
    .category-badges {
      display: flex;
      gap: 0.75rem;
      margin: 1.5rem 0;
      flex-wrap: wrap;
    }
    
    .badge {
      padding: 0.5rem 1rem;
      border-radius: 20px;
      font-size: 0.9rem;
      font-weight: 500;
      transition: all 0.2s;
    }
    
    .badge-security {
      background: linear-gradient(135deg, #ef4444, #dc2626);
      color: white;
      box-shadow: 0 2px 8px rgba(239, 68, 68, 0.3);
    }
    
    .badge-structure {
      background: linear-gradient(135deg, #3b82f6, #2563eb);
      color: white;
      box-shadow: 0 2px 8px rgba(59, 130, 246, 0.3);
    }
    
    .badge-quality {
      background: linear-gradient(135deg, #22c55e, #16a34a);
      color: white;
      box-shadow: 0 2px 8px rgba(34, 197, 94, 0.3);
    }
    
    .badge-disabled {
      background: var(--color-card);
      color: var(--color-dim);
      opacity: 0.5;
    }
    
    .verdict {
      text-align: center;
      padding: 2rem;
      border-radius: 12px;
      margin-bottom: 2rem;
    }
    
    .verdict.pass { background: rgba(34, 197, 94, 0.2); border: 2px solid var(--color-success); }
    .verdict.fail { background: rgba(239, 68, 68, 0.2); border: 2px solid var(--color-error); }
    
    .verdict-text {
      font-size: 1.5rem;
      font-weight: bold;
    }
    
    .violations-section {
      margin-bottom: 2rem;
    }
    
    .section-title {
      font-size: 1.25rem;
      margin-bottom: 1rem;
      padding-bottom: 0.5rem;
      border-bottom: 1px solid var(--color-card);
    }
    
    .file-group {
      margin-bottom: 1.5rem;
    }
    
    .file-name {
      font-weight: bold;
      color: var(--color-dim);
      margin-bottom: 0.5rem;
    }
    
    .violation {
      background: var(--color-card);
      border-radius: 8px;
      padding: 1rem;
      margin-bottom: 0.75rem;
      border-left: 4px solid;
    }
    
    .violation.severity-error { border-color: var(--color-error); }
    .violation.severity-warning { border-color: var(--color-warning); }
    .violation.severity-info { border-color: var(--color-info); }
    
    .violation-header {
      display: flex;
      flex-wrap: wrap;
      gap: 0.5rem;
      align-items: center;
      margin-bottom: 0.5rem;
    }
    
    .rule-id {
      font-weight: bold;
      font-family: monospace;
    }
    
    .rule-name { color: var(--color-dim); }
    
    .location {
      font-family: monospace;
      font-size: 0.875rem;
      color: var(--color-dim);
      margin-left: auto;
    }
    
    .violation-message { margin-bottom: 0.5rem; }
    
    .snippet {
      background: rgba(0, 0, 0, 0.3);
      padding: 0.5rem;
      border-radius: 4px;
      font-family: monospace;
      font-size: 0.875rem;
      overflow-x: auto;
      margin-bottom: 0.5rem;
    }
    
    .suggestion {
      color: #06b6d4;
      font-size: 0.875rem;
    }
    
    .score-section {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
      gap: 1rem;
      margin-bottom: 2rem;
    }
    
    .score-bar {
      height: 8px;
      background: rgba(255, 255, 255, 0.1);
      border-radius: 4px;
      overflow: hidden;
      margin-top: 0.5rem;
    }
    
    .score-fill {
      height: 100%;
      background: linear-gradient(90deg, #06b6d4, #3b82f6);
      transition: width 0.3s;
    }
    
    .grade {
      font-size: 4rem;
      font-weight: bold;
      text-align: center;
    }
    
    .grade-A { color: var(--color-success); }
    .grade-B { color: var(--color-info); }
    .grade-C { color: var(--color-warning); }
    .grade-D { color: var(--color-error); }
    
    .gate-status {
      display: grid;
      grid-template-columns: repeat(4, 1fr);
      gap: 0.5rem;
    }
    
    .gate {
      padding: 1rem;
      text-align: center;
      border-radius: 8px;
    }
    
    .gate.pass { background: rgba(34, 197, 94, 0.2); }
    .gate.fail { background: rgba(239, 68, 68, 0.2); }
    
    @media (max-width: 768px) {
      body { padding: 1rem; }
      .gate-status { grid-template-columns: repeat(2, 1fr); }
    }
  </style>
</head>
<body>
  <div class="container">
    <h1>📊 MAIDOS CodeQC Report</h1>
    <div class="meta">
      <div>📂 ${escapeHtml(result.targetPath)}</div>
      <div>📊 Level: ${result.level} | ⏱️ ${result.duration}ms | 📁 ${summary.totalFiles} files</div>
      <div>🕐 ${result.timestamp}</div>
      ${result.categories ? `<div class="analysis-categories">🔍 分析類型: ${renderCategories(result.categories)}</div>` : ''}
    </div>
    
    ${result.categories ? `
    <div class="category-badges">
      ${result.categories.includes('security') ? '<span class="badge badge-security">🔒 安全性 Security</span>' : '<span class="badge badge-disabled">🔒 安全性</span>'}
      ${result.categories.includes('structure') ? '<span class="badge badge-structure">🏗️ 結構性 Structure</span>' : '<span class="badge badge-disabled">🏗️ 結構性</span>'}
      ${result.categories.includes('quality') ? '<span class="badge badge-quality">✨ 代碼質量 Quality</span>' : '<span class="badge badge-disabled">✨ 代碼質量</span>'}
    </div>
    ` : ''}
    
    <div class="verdict ${passed ? 'pass' : 'fail'}">
      <div class="verdict-text">${passed ? '✅ Gate-Out: PASS' : '❌ Gate-Out: FAIL'}</div>
    </div>
    
    <div class="summary-grid">
      <div class="card">
        <div class="card-title">Total Violations</div>
        <div class="card-value ${summary.totalViolations > 0 ? 'error' : 'success'}">${summary.totalViolations}</div>
      </div>
      <div class="card">
        <div class="card-title">Errors</div>
        <div class="card-value ${summary.errorCount > 0 ? 'error' : 'success'}">${summary.errorCount}</div>
      </div>
      <div class="card">
        <div class="card-title">Warnings</div>
        <div class="card-value ${summary.warningCount > 0 ? 'warning' : 'success'}">${summary.warningCount}</div>
      </div>
      <div class="card">
        <div class="card-title">Info</div>
        <div class="card-value info">${summary.infoCount}</div>
      </div>
    </div>
    
    ${score ? `
    <div class="section-title">📈 Dual-Axis Score</div>
    <div class="score-section">
      <div class="card">
        <div class="card-title">X-Axis: Compliance</div>
        <div class="card-value">${score.x.total}%</div>
        <div class="score-bar"><div class="score-fill" style="width: ${score.x.total}%"></div></div>
      </div>
      <div class="card">
        <div class="card-title">Y-Axis: Outcome</div>
        <div class="card-value">${score.y.total}%</div>
        <div class="score-bar"><div class="score-fill" style="width: ${score.y.total}%"></div></div>
      </div>
      <div class="card">
        <div class="card-title">Grade</div>
        <div class="grade grade-${score.grade}">${score.grade}</div>
      </div>
    </div>
    ` : ''}
    
    ${gates ? `
    <div class="section-title">🚪 Gate Status</div>
    <div class="card">
      <div class="gate-status">
        <div class="gate ${gates.gateIn.passed ? 'pass' : 'fail'}">
          <div>${gates.gateIn.passed ? '✅' : '❌'}</div>
          <div>Gate-In</div>
        </div>
        <div class="gate ${gates.gateMid.passed ? 'pass' : 'fail'}">
          <div>${gates.gateMid.passed ? '✅' : '❌'}</div>
          <div>Gate-Mid</div>
        </div>
        <div class="gate ${gates.gateOut.passed ? 'pass' : 'fail'}">
          <div>${gates.gateOut.passed ? '✅' : '❌'}</div>
          <div>Gate-Out</div>
        </div>
        <div class="gate ${gates.gateAccept.passed ? 'pass' : 'fail'}">
          <div>${gates.gateAccept.passed ? '✅' : '❌'}</div>
          <div>Gate-Accept</div>
        </div>
      </div>
    </div>
    ` : ''}
    
    ${summary.totalViolations > 0 ? `
    <div class="violations-section">
      <div class="section-title">⚠️ Violations</div>
      ${result.files
        .filter(f => f.violations.length > 0)
        .map(f => `
          <div class="file-group">
            <div class="file-name">📄 ${escapeHtml(f.file)}</div>
            ${f.violations.map(v => renderViolation(v)).join('')}
          </div>
        `).join('')}
    </div>
    ` : ''}
    
    <div class="meta" style="text-align: center; margin-top: 3rem;">
      Generated by MAIDOS CodeQC v2.4
    </div>
  </div>
</body>
</html>`;
  },
};

export default htmlReporter;
