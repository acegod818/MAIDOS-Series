/**
 * MAIDOS CodeQC v3.3 — CLI Commands
 *
 * Pipeline + Serve 子命令實作
 * 從 cli.ts 拆出，避免單檔超長
 */

// =============================================================================
// External Command Runner (Pipeline 外部注入)
// =============================================================================

interface ExternalResult {
  exitCode: number;
  log: string;
}

function stripAnsi(input: string): string {
  // Remove common ANSI SGR sequences (colors/styles) so regex parsing stays stable.
  // eslint-disable-next-line no-control-regex
  return input.replace(/\x1b\[[0-9;]*m/g, '');
}

async function runExternalCmd(cmd: string, cwd: string): Promise<ExternalResult> {
  const { execSync } = await import('node:child_process');
  try {
    const log = execSync(cmd, { cwd, encoding: 'utf-8', timeout: 120_000, stdio: ['pipe', 'pipe', 'pipe'] });
    return { exitCode: 0, log };
  } catch (err: unknown) {
    const e = (err && typeof err === 'object') ? (err as Record<string, unknown>) : {};
    const exitCode = typeof e.status === 'number' ? e.status : 1;
    const stdout = e.stdout != null ? String(e.stdout) : '';
    const stderr = e.stderr != null ? String(e.stderr) : '';
    return { exitCode, log: stdout + '\n' + stderr };
  }
}

export function parseTestOutput(log: string): { passed: number; failed: number } {
  const clean = stripAnsi(log);

  // vitest format: "Tests  167 passed (167)" — prefer "Tests" line over "Test Files" line.
  // Some projects may log nested vitest output (e.g. running a pipeline in tests),
  // so we take the maximum to avoid being tricked by a smaller inner run.
  const vitestPass = [...clean.matchAll(/Tests\s+(\d+)\s*passed/gi)].map(m => parseInt(m[1]!, 10));
  const vitestFail = [...clean.matchAll(/Tests\s+(\d+)\s*failed/gi)].map(m => parseInt(m[1]!, 10));
  if (vitestPass.length > 0) {
    return {
      passed: Math.max(...vitestPass),
      failed: vitestFail.length > 0 ? Math.max(...vitestFail) : 0,
    };
  }
  // jest format: "Tests: X passed, Y failed"
  const jestPass = clean.match(/Tests:\s*.*?(\d+)\s*passed/i);
  const jestFail = clean.match(/Tests:\s*.*?(\d+)\s*failed/i);
  if (jestPass) {
    return {
      passed: parseInt(jestPass[1]!, 10),
      failed: jestFail ? parseInt(jestFail[1]!, 10) : 0,
    };
  }
  // generic fallback
  const allPass = [...clean.matchAll(/(\d+)\s*pass(?:ed)?/gi)].map(m => parseInt(m[1]!, 10));
  const allFail = [...clean.matchAll(/(\d+)\s*fail(?:ed)?/gi)].map(m => parseInt(m[1]!, 10));
  return {
    passed: allPass.length > 0 ? Math.max(...allPass) : 0,
    failed: allFail.length > 0 ? Math.max(...allFail) : 0,
  };
}

export function parseCoverage(log: string): number {
  const tableMatch = log.match(/All files\s*\|\s*([\d.]+)/);
  if (tableMatch) return parseFloat(tableMatch[1]!);
  const stmtMatch = log.match(/(?:Statements|Lines|Branches)\s*:\s*([\d.]+)%/i);
  if (stmtMatch) return parseFloat(stmtMatch[1]!);
  const pctMatch = log.match(/([\d.]+)%\s*(?:coverage|covered)/i);
  if (pctMatch) return parseFloat(pctMatch[1]!);
  const v8Match = log.match(/All files[^|]*\|\s*([\d.]+)\s*\|/);
  if (v8Match) return parseFloat(v8Match[1]!);
  // cargo llvm-cov --summary-only table
  // Example tail:
  // TOTAL ... 37.32% ... 40.53% ... 38.26% ... -
  // We prefer the "Lines Cover" column, falling back to the last %-token.
  const lines = log.split(/\r?\n/);
  const headerIdx = lines.findIndex(l => l.trim().startsWith('Filename') && l.includes('Regions') && l.includes('Missed'));
  const totalIdx = lines.findIndex(l => l.trim().startsWith('TOTAL'));
  if (headerIdx !== -1 && totalIdx !== -1) {
    const headerParts = lines[headerIdx]!.trim().split(/\s{2,}/);
    const coverIdxs = headerParts
      .map((p, i) => ({ p, i }))
      .filter(x => x.p === 'Cover')
      .map(x => x.i);
    const preferredCoverIdx = coverIdxs.length >= 2 ? coverIdxs[1]! : coverIdxs[0];
    if (preferredCoverIdx != null) {
      const totalParts = lines[totalIdx]!.trim().split(/\s{2,}/);
      const token = totalParts[preferredCoverIdx] ?? '';
      const m = token.match(/([\d.]+)%/);
      if (m) return parseFloat(m[1]!);
    }
  }
  const totalLine = log.match(/^TOTAL.*$/m)?.[0];
  if (totalLine) {
    const matches = [...totalLine.matchAll(/([\d.]+)%/g)];
    if (matches.length > 0) return parseFloat(matches[matches.length - 1]![1]!);
  }
  return 0;
}

export function parseAudit(log: string): { critical: number; high: number } {
  // Prefer npm audit --json output: metadata.vulnerabilities.{critical,high}
  try {
    const j: unknown = JSON.parse(log);
    if (j && typeof j === 'object') {
      const metadata = (j as { metadata?: unknown }).metadata;
      if (metadata && typeof metadata === 'object') {
        const vulns = (metadata as { vulnerabilities?: unknown }).vulnerabilities;
        if (vulns && typeof vulns === 'object') {
          const critical = (vulns as { critical?: unknown }).critical;
          const high = (vulns as { high?: unknown }).high;
          return {
            critical: typeof critical === 'number' ? critical : 0,
            high: typeof high === 'number' ? high : 0,
          };
        }
      }
    }
  } catch { /* ignore */ }

  // Fallback heuristic (still better than silently passing).
  return {
    critical: (log.match(/\bcritical\b/gi) || []).length,
    high: (log.match(/\bhigh\b/gi) || []).length,
  };
}

export function parseIavLog(content: string): { passed: boolean; passedCount: number; failedCount: number; details: string } {
  // NOTE: Avoid `\\b` word-boundaries here (Chinese text isn't a "word" in JS regex terms).
  const verdicts = [...content.matchAll(/判定\s*:\s*(PASS|FAIL)/gi)].map(m => m[1]!.toUpperCase());
  const passedCount = verdicts.filter(v => v === 'PASS').length;
  const failedCount = verdicts.filter(v => v === 'FAIL').length;

  // Minimal sanity: Q1-Q5 must appear at least once.
  const qOk = ['Q1', 'Q2', 'Q3', 'Q4', 'Q5'].every(q => new RegExp(`\\b${q}\\b`).test(content));
  const passed = verdicts.length > 0 && failedCount === 0 && passedCount > 0 && qOk;

  return {
    passed,
    passedCount,
    failedCount,
    details: passed
      ? `IAV PASS (${passedCount} points)`
      : `IAV FAIL (points=${verdicts.length}, pass=${passedCount}, fail=${failedCount}, qOk=${qOk})`,
  };
}

export function parseBldsLog(content: string, threshold: number): { passed: boolean; minScore: number; threshold: number; details: string } {
  const totals: number[] = [];
  for (const m of content.matchAll(/總分[^0-9]*([0-5])\s*\/\s*5/gi)) {
    totals.push(parseInt(m[1]!, 10));
  }
  for (const m of content.matchAll(/\bBLDS\b\s*[:=]\s*([0-5])\b/gi)) {
    totals.push(parseInt(m[1]!, 10));
  }

  const minScore = totals.length > 0 ? Math.min(...totals) : 0;
  const passed = totals.length > 0 && minScore >= threshold;

  return {
    passed,
    minScore,
    threshold,
    details: passed
      ? `BLDS PASS (min=${minScore}/5 >= ${threshold})`
      : `BLDS FAIL (min=${minScore}/5, need>=${threshold}, points=${totals.length})`,
  };
}

export function parseDatasourceLog(content: string): { passed: boolean; untraced: number; details: string } {
  const lines = content.split(/\r?\n/).map(l => l.trim()).filter(Boolean);
  const untraced = lines.filter(l => /^(UNTRACED|MISSING)\b/i.test(l) || /未追溯|未追蹤/i.test(l)).length;
  const passed = lines.length > 0 && untraced === 0;
  return {
    passed,
    untraced,
    details: passed
      ? `DATASOURCE PASS (lines=${lines.length})`
      : `DATASOURCE FAIL (lines=${lines.length}, untraced=${untraced})`,
  };
}

export async function detectPackageScripts(targetPath: string): Promise<Record<string, string>> {
  const { readFileSync, existsSync } = await import('node:fs');
  const { resolve } = await import('node:path');
  const scripts: Record<string, string> = {};
  const pkgPath = resolve(targetPath, 'package.json');

  if (existsSync(pkgPath)) {
    try {
      const pkg = JSON.parse(readFileSync(pkgPath, 'utf-8'));
      const s = pkg.scripts || {};
      if (s.typecheck) scripts.build = 'npm run typecheck';
      else if (s.build) scripts.build = 'npm run build';
      if (s.typecheck) scripts.lint = 'npm run typecheck';
      else if (s.lint) scripts.lint = 'npm run lint';
      if (s.test) {
        const testVal = s.test as string;
        if (testVal.includes('vitest') && !testVal.includes('run')) {
          scripts.test = 'npx vitest run';
        } else if (testVal.includes('jest') && !testVal.includes('--forceExit')) {
          scripts.test = 'npx jest --forceExit';
        } else {
          scripts.test = 'npm test';
        }
      }
      if (s['test:coverage']) scripts.coverage = 'npm run test:coverage';
      else if (s.coverage) scripts.coverage = 'npm run coverage';
      else if (s.test) {
        const testVal = s.test as string;
        if (testVal.includes('vitest')) scripts.coverage = 'npx vitest run --coverage';
        else if (testVal.includes('jest')) scripts.coverage = 'npx jest --coverage --forceExit';
      }
    } catch { /* ignore */ }
  }
  return scripts;
}

/** `maidos-codeqc serve` — Start SaaS API server */
export async function serveCommand(args: string[]) {
  const { startServer } = await import('./server/app.js');

  // Keep subcommand UX predictable (avoid accidentally starting a server).
  if (args.includes('-h') || args.includes('--help')) {
    console.log(`
MAIDOS CodeQC serve

Usage:
  maidos-codeqc serve [options]

Options:
  -p, --port <port>     Listen port (default: 3333)
  --api-key <key>       Optional API key for requests
  -h, --help            Show this help
`);
    return;
  }

  let port = 3333;
  let apiKey: string | undefined;

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--port' || args[i] === '-p') port = parseInt(args[++i] || '3333', 10);
    if (args[i] === '--api-key') apiKey = args[++i];
  }

  await startServer({ port, apiKey });
}

// loadFiles type — imported by pipelineCommand caller
type FileEntry = { path: string; content: string };
type LoadFilesFn = (target: string) => FileEntry[];

/** `maidos-codeqc pipeline` — Run full v3.3 pipeline */
export async function pipelineCommand(args: string[], loadFiles: LoadFilesFn) {
  const { runPipeline, formatPipelineReport } = await import('./engine/pipeline.js');
  const { generateProofPackManifest, collectEvidence } = await import('./engine/evidence.js');
  const { writeFileSync, mkdirSync, existsSync } = await import('node:fs');
  const { resolve } = await import('node:path');
  type ProductGrade = 'E' | 'F';

  if (args.includes('-h') || args.includes('--help')) {
    console.log(`
MAIDOS CodeQC pipeline (v3.3)

Usage:
  maidos-codeqc pipeline [target] [options]

Options:
  -g, --grade <E|F>         Product grade (default: E)
  --build-cmd <cmd>         Build command to execute inside target
  --lint-cmd <cmd>          Lint/typecheck command
  --test-cmd <cmd>          Test command
  --coverage-cmd <cmd>      Coverage command (must print coverage % somewhere)
  --audit-cmd <cmd>         Security audit command (optional; prefer JSON output)
  --package-cmd <cmd>       Packaging command (required for DoD#6)
  --run-cmd <cmd>           Run command (required for DoD#6, e.g. --version)
  --spec <path>             Spec file path under target (optional)
  --no-auto                 Disable auto-detect (package.json scripts)
  -h, --help                Show this help

Examples:
  maidos-codeqc pipeline . --build-cmd "cargo build" --test-cmd "cargo test"
  maidos-codeqc pipeline . --no-auto --build-cmd "dotnet build" --test-cmd "dotnet test"
`);
    return;
  }

  let target = '.';
  let grade: ProductGrade = 'E';
  let buildCmd: string | undefined;
  let testCmd: string | undefined;
  let lintCmd: string | undefined;
  let coverageCmd: string | undefined;
  let auditCmd: string | undefined;
  let packageCmd: string | undefined;
  let runCmd: string | undefined;
  let autoDetect = true;
  let specPath: string | undefined;

  for (let i = 0; i < args.length; i++) {
    const arg = args[i]!;
    if (arg === '--grade' || arg === '-g') {
      const g = args[++i]?.toUpperCase();
      if (g === 'E' || g === 'F') grade = g;
    } else if (arg === '--build-cmd') { buildCmd = args[++i]; }
    else if (arg === '--test-cmd') { testCmd = args[++i]; }
    else if (arg === '--lint-cmd') { lintCmd = args[++i]; }
    else if (arg === '--coverage-cmd') { coverageCmd = args[++i]; }
    else if (arg === '--audit-cmd') { auditCmd = args[++i]; }
    else if (arg === '--package-cmd') { packageCmd = args[++i]; }
    else if (arg === '--run-cmd') { runCmd = args[++i]; }
    else if (arg === '--spec') { specPath = args[++i]; }
    else if (arg === '--no-auto') { autoDetect = false; }
    else if (arg === '--full') { /* legacy compat */ }
    else if (!arg.startsWith('-')) { target = arg; }
  }

  const absTarget = resolve(target);
  console.log(`\n🔧 CodeQC v3.3 Pipeline — Grade ${grade}`);
  console.log(`   Target: ${absTarget}\n`);

  // ── Auto-detect scripts from package.json ──
  if (autoDetect) {
    const detected = await detectPackageScripts(absTarget);
    if (!buildCmd && detected.build) buildCmd = detected.build;
    if (!testCmd && detected.test) testCmd = detected.test;
    if (!lintCmd && detected.lint) lintCmd = detected.lint;
    if (!coverageCmd && detected.coverage) coverageCmd = detected.coverage;
  }

  // ── Run external commands ──
  const externalResults: NonNullable<import('./engine/pipeline.js').PipelineInput['externalResults']> = {};

  if (buildCmd) {
    console.log(`  🔨 Build: ${buildCmd}`);
    const r = await runExternalCmd(buildCmd, absTarget);
    externalResults.build = { exitCode: r.exitCode, log: r.log };
    console.log(`     → exit ${r.exitCode}\n`);
  }

  if (lintCmd) {
    console.log(`  🧹 Lint: ${lintCmd}`);
    const r = await runExternalCmd(lintCmd, absTarget);
    externalResults.lint = { exitCode: r.exitCode, log: r.log };
    console.log(`     → exit ${r.exitCode}\n`);
  }

  if (testCmd) {
    console.log(`  🧪 Test: ${testCmd}`);
    const r = await runExternalCmd(testCmd, absTarget);
    const parsed = parseTestOutput(r.log);
    externalResults.test = { exitCode: r.exitCode, log: r.log, ...parsed };
    console.log(`     → exit ${r.exitCode} (${parsed.passed} passed, ${parsed.failed} failed)\n`);
  }

  if (coverageCmd) {
    console.log(`  📊 Coverage: ${coverageCmd}`);
    const r = await runExternalCmd(coverageCmd, absTarget);
    const pct = parseCoverage(r.log);
    externalResults.coverage = { percentage: pct, log: r.log };
    console.log(`     → ${pct}%\n`);
  }

  if (auditCmd) {
    console.log(`  🔎 Audit: ${auditCmd}`);
    const r = await runExternalCmd(auditCmd, absTarget);
    const parsed = parseAudit(r.log);
    externalResults.audit = { exitCode: r.exitCode, log: r.log, ...parsed };
    console.log(`     → exit ${r.exitCode} (critical=${parsed.critical}, high=${parsed.high})\n`);
  }

  if (packageCmd) {
    console.log(`  📦 Package: ${packageCmd}`);
    const r = await runExternalCmd(packageCmd, absTarget);
    externalResults.package = { exitCode: r.exitCode, log: r.log };
    console.log(`     → exit ${r.exitCode}\n`);
  }

  if (runCmd) {
    console.log(`  ⚡ Run: ${runCmd}`);
    const r = await runExternalCmd(runCmd, absTarget);
    externalResults.run = { exitCode: r.exitCode, log: r.log };
    console.log(`     → exit ${r.exitCode}\n`);
  }

  // ── Discover files ──
  const rawFiles = loadFiles(target);
  const files = rawFiles.map(f => ({ path: f.path, content: f.content }));

  // ── SPEC functions ──
  let specFunctions: string[] | undefined;
  let specChecklist: { total: number; done: number } | undefined;
  if (specPath && existsSync(resolve(absTarget, specPath))) {
    const specContent = (await import('node:fs')).readFileSync(resolve(absTarget, specPath), 'utf-8');
    specFunctions = [...specContent.matchAll(/→\s*`([^`]+)`/g)].map(m => m[1]!);
    const total = (specContent.match(/^\s*-\s*\[\s*[xX ]\s*\]/gm) || []).length;
    const done = (specContent.match(/^\s*-\s*\[\s*[xX]\s*\]/gm) || []).length;
    if (total > 0) specChecklist = { total, done };
  }

  // ── Evidence directory ──
  const evidenceDir = resolve(absTarget, 'evidence');
  try { mkdirSync(evidenceDir, { recursive: true }); } catch { /* exists */ }

  // ── Z 軸真實性證據 (IAV/BLDS/DATASOURCE) ──
  let proof: import('./engine/pipeline.js').PipelineInput['proof'] | undefined;
  const iavPath = resolve(evidenceDir, 'iav.log');
  const bldsPath = resolve(evidenceDir, 'blds.log');
  const dsPath = resolve(evidenceDir, 'datasource.log');

  const iavContent = existsSync(iavPath) ? (await import('node:fs')).readFileSync(iavPath, 'utf-8') : undefined;
  const bldsContent = existsSync(bldsPath) ? (await import('node:fs')).readFileSync(bldsPath, 'utf-8') : undefined;
  const dsContent = existsSync(dsPath) ? (await import('node:fs')).readFileSync(dsPath, 'utf-8') : undefined;

  const bldsThreshold = grade === 'F' ? 4 : 3;
  const proofContent: import('./engine/pipeline.js').PipelineInput['proofContent'] = {
    iavLog: iavContent,
    bldsLog: bldsContent,
    datasourceLog: dsContent,
  };
  if (iavContent || bldsContent || dsContent) {
    proof = {};
    if (iavContent) {
      const p = parseIavLog(iavContent);
      proof.iav = { passed: p.passed, passedCount: p.passedCount, failedCount: p.failedCount, details: p.details };
    }
    if (bldsContent) {
      const p = parseBldsLog(bldsContent, bldsThreshold);
      proof.blds = { minScore: p.minScore, threshold: p.threshold, passed: p.passed, details: p.details };
    }
    if (dsContent) {
      const p = parseDatasourceLog(dsContent);
      proof.datasource = { untraced: p.untraced, passed: p.passed, details: p.details };
    }
  }

  // ── Run Pipeline ──
  const pipelineResult = runPipeline({
    targetPath: target,
    files,
    grade,
    evidenceDir,
    externalResults: Object.keys(externalResults).length > 0 ? externalResults : undefined,
    specPath,
    specFunctions,
    specChecklist,
    proof,
    proofContent,
  });

  // ── Print report ──
  console.log(formatPipelineReport(pipelineResult));

  // ── Write evidence logs ──
  for (const [name, data] of Object.entries(externalResults)) {
    const val = data as ExternalResult & { passed?: number; failed?: number; percentage?: number };
    if (val.log) {
      writeFileSync(resolve(evidenceDir, `${name}.log`), val.log, 'utf-8');
    }
  }

  // Internal pipeline evidence (scan/fraud/redline/sync/mapping/g4)
  for (const step of pipelineResult.steps) {
    if (!step.evidencePath) continue;
    if (typeof step.log !== 'string') continue;
    // step.evidencePath is like "evidence/scan.log"
    const p = resolve(absTarget, step.evidencePath);
    writeFileSync(p, step.log, 'utf-8');
  }

  // Gate summary logs (g1-g4)
  writeFileSync(resolve(evidenceDir, 'g1.log'), `G1 ${pipelineResult.gates.G1.passed ? 'PASS' : 'FAIL'}\n${pipelineResult.gates.G1.details}\n`, 'utf-8');
  writeFileSync(resolve(evidenceDir, 'g2.log'), `G2 ${pipelineResult.gates.G2.passed ? 'PASS' : 'FAIL'}\n${pipelineResult.gates.G2.details}\n`, 'utf-8');
  writeFileSync(resolve(evidenceDir, 'g3.log'), `G3 ${pipelineResult.gates.G3.passed ? 'PASS' : 'FAIL'}\n${pipelineResult.gates.G3.details}\n`, 'utf-8');
  writeFileSync(resolve(evidenceDir, 'g4.log'), `G4 ${pipelineResult.gates.G4.passed ? 'PASS' : 'FAIL'}\n${pipelineResult.gates.G4.details}\n`, 'utf-8');

  // impl.log — 補完證明 (對齊 DoD#2 的 evidence 名稱)
  const implLines: string[] = [];
  if (specChecklist) {
    const pct = specChecklist.total > 0 ? Math.round((specChecklist.done / specChecklist.total) * 100) : 0;
    implLines.push(`SPEC: ${pct}% (${specChecklist.done}/${specChecklist.total})`);
  } else {
    implLines.push('SPEC: (no checklist parsed)');
  }
  const missingCount = pipelineResult.steps[8]?.stats?.missing ?? 0;
  implLines.push(`MissingFunctions: ${missingCount}`);
  implLines.push(pipelineResult.steps[8]?.passed ? 'OK: All SPEC functions are implemented' : 'FAIL: SPEC gaps remain (see mapping.log)');
  writeFileSync(resolve(evidenceDir, 'impl.log'), implLines.join('\n') + '\n', 'utf-8');

  // LV4/LV5 evidence (anti-replay + anti-tamper)
  writeFileSync(resolve(evidenceDir, 'nonce.log'), pipelineResult.nonce + '\n', 'utf-8');
  writeFileSync(resolve(evidenceDir, 'hash.log'), pipelineResult.evidenceHash + '\n', 'utf-8');

  // ── Write Proof Pack manifest ──
  const evidence = collectEvidence(pipelineResult.steps, pipelineResult.gates, { evidenceDir, files, externalResults, specChecklist, proof });
  const manifest = generateProofPackManifest(evidence);
  const reportPath = resolve(evidenceDir, 'PROOF_PACK.md');
  writeFileSync(reportPath, manifest, 'utf-8');
  console.log(`  📄 Proof Pack: ${reportPath}\n`);

  // Minimal PROOF-REPORT.md (waveform snapshot)
  const proofReportLines: string[] = [];
  proofReportLines.push('# Proof Report v3.3');
  proofReportLines.push(`- Target: ${absTarget}`);
  proofReportLines.push(`- Timestamp: ${pipelineResult.timestamp}`);
  proofReportLines.push(`- Verdict: ${pipelineResult.passed ? 'MISSION COMPLETE ✅' : 'REJECTED ❌'}`);
  proofReportLines.push('');

  if (pipelineResult.waveform) {
    proofReportLines.push('## Waveform (三通道示波器)');
    proofReportLines.push('');
    for (const ch of pipelineResult.waveform.channels) {
      proofReportLines.push(`### ${ch.channel} ${ch.name} — ${ch.overall} (score=${ch.score})`);
      proofReportLines.push('');
      proofReportLines.push('| ID | Reading | Status | Evidence |');
      proofReportLines.push('|:--:|:--------|:------:|:---------|');
      for (const r of ch.readings) {
        proofReportLines.push(`| ${r.id} | ${r.name} | ${r.status} | ${r.evidence ?? ''} |`);
      }
      proofReportLines.push('');
    }
    proofReportLines.push(`Composite Score: ${pipelineResult.waveform.compositeScore}`);
    proofReportLines.push('');
  }

  writeFileSync(resolve(evidenceDir, 'PROOF-REPORT.md'), proofReportLines.join('\n'), 'utf-8');

  if (!pipelineResult.passed) process.exit(1);
}
