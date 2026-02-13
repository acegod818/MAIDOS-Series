import { R25_CHECKER, R26_CHECKER, R27_CHECKER, R28_CHECKER } from './src/rules/b-redlines-r25-r28.js';
import { stripRustCfgTestBlocks } from './src/rules/b-redlines-utils.js';
import * as fs from 'fs';
import * as path from 'path';

const CHECKERS = [R25_CHECKER, R26_CHECKER, R27_CHECKER, R28_CHECKER];
const ROOT = path.resolve('../../');
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
  } catch (e) { console.error(`[rescan:walk] Failed to read directory ${dir}: ${e}`); }
  return files;
}

const files = walk(ROOT);
const all: Array<{ rule: string; file: string; line: number; fn: string }> = [];

for (const f of files) {
  const rel = path.relative(ROOT, f);
  try {
    let source = fs.readFileSync(f, 'utf-8');
    const ext = f.split('.').pop()?.toLowerCase() || '';
    if (ext === 'rs') source = stripRustCfgTestBlocks(source);
    for (const checker of CHECKERS) {
      const violations = checker.checkSource?.(source, rel) ?? [];
      for (const v of violations) {
        stats[v.ruleId]!++;
        const fnMatch = v.message.match(/fn (\w+)\(/);
        all.push({ rule: v.ruleId, file: rel, line: v.line, fn: fnMatch?.[1] ?? '' });
      }
    }
  } catch (e) { console.error(`[rescan] Failed to process ${rel}: ${e}`); }
}

for (const rule of ['R25', 'R26', 'R27', 'R28']) {
  const rv = all.filter(v => v.rule === rule);
  console.log(`\n=== ${rule} (${rv.length}) ===`);
  for (const v of rv) console.log(`${v.file}:${v.line}:${v.fn}`);
}
console.log(`\nTOTAL: ${Object.values(stats).reduce((a, b) => a + b, 0)}`);
