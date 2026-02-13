import { R25_CHECKER, R26_CHECKER, R27_CHECKER, R28_CHECKER } from './src/rules/b-redlines-r25-r28.js';
import * as fs from 'fs';
import * as path from 'path';

const CHECKERS = [R25_CHECKER, R26_CHECKER, R27_CHECKER, R28_CHECKER];
const ROOT = path.resolve('../../');  // MAIDOS-Series root
const stats: Record<string, number> = { R25: 0, R26: 0, R27: 0, R28: 0 };

function walk(dir: string): string[] {
  const files: string[] = [];
  try {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name);
      if (entry.name === 'node_modules' || entry.name === '.git' || entry.name === 'target') continue;
      if (entry.isDirectory()) files.push(...walk(full));
      else if (/\.(rs|cs|cpp|h)$/.test(entry.name)) files.push(full);
    }
  } catch (e) { console.error(`[walk] Failed to read directory ${dir}: ${e}`); }
  return files;
}

const files = walk(ROOT);
const allViolations: Array<{ rule: string; file: string; line: number; msg: string }> = [];

for (const f of files) {
  const rel = path.relative(ROOT, f);
  try {
    const source = fs.readFileSync(f, 'utf-8');
    for (const checker of CHECKERS) {
      const violations = checker.checkSource?.(source, rel) ?? [];
      for (const v of violations) {
        stats[v.ruleId]!++;
        allViolations.push({ rule: v.ruleId, file: rel, line: v.line, msg: v.message.substring(0, 70) });
      }
    }
  } catch (e) { console.error(`[scan] Failed to process ${rel}: ${e}`); }
}

console.log('╔════════════════════════════════════════════════════╗');
console.log('║  R25-R28 Codebase Scan Results                    ║');
console.log('╠════════════════════════════════════════════════════╣');
for (const [rule, count] of Object.entries(stats)) {
  console.log(`║  ${rule}: ${count} violations`);
}
console.log(`║  Total: ${Object.values(stats).reduce((a, b) => a + b, 0)} violations`);
console.log('╠════════════════════════════════════════════════════╣');

// Show first few of each
for (const rule of ['R25', 'R26', 'R27', 'R28']) {
  const ruleViolations = allViolations.filter(v => v.rule === rule);
  if (ruleViolations.length > 0) {
    console.log(`\n║ ${rule} samples:`);
    for (const v of ruleViolations.slice(0, 5)) {
      console.log(`║   ${v.file}:${v.line}`);
    }
    if (ruleViolations.length > 5) console.log(`║   ... +${ruleViolations.length - 5} more`);
  }
}
console.log('╚════════════════════════════════════════════════════╝');
