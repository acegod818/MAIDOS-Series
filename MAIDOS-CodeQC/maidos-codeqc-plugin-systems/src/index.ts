/**
 * @maidos/codeqc-plugin-systems
 * Systems programming languages: C, C++, Zig, Nim
 */

import type { Violation } from '@maidos/codeqc';

export { C_CONFIG, checkC } from './languages/c.js';
export { CPP_CONFIG, checkCpp } from './languages/cpp.js';
export { ZIG_CONFIG, checkZig } from './languages/zig.js';
export { NIM_CONFIG, checkNim } from './languages/nim.js';

import { C_CONFIG, checkC } from './languages/c.js';
import { CPP_CONFIG, checkCpp } from './languages/cpp.js';
import { ZIG_CONFIG, checkZig } from './languages/zig.js';
import { NIM_CONFIG, checkNim } from './languages/nim.js';

export interface SystemsLanguageSupport {
  id: string;
  name: string;
  extensions: string[];
  check: (source: string, file: string) => Violation[];
}

export const LANGUAGES: SystemsLanguageSupport[] = [
  { ...C_CONFIG, check: checkC },
  { ...CPP_CONFIG, check: checkCpp },
  { ...ZIG_CONFIG, check: checkZig },
  { ...NIM_CONFIG, check: checkNim },
];

export function getLanguageByExtension(file: string): SystemsLanguageSupport | undefined {
  const ext = '.' + file.split('.').pop()?.toLowerCase();
  return LANGUAGES.find(lang => lang.extensions.includes(ext));
}

export function checkSystemsFile(source: string, file: string): Violation[] {
  const lang = getLanguageByExtension(file);
  return lang ? lang.check(source, file) : [];
}

export function isSystemsFile(file: string): boolean {
  return getLanguageByExtension(file) !== undefined;
}

export const plugin = {
  name: '@maidos/codeqc-plugin-systems',
  version: '0.1.0',
  languages: LANGUAGES.map(lang => ({
    id: lang.id,
    extensions: lang.extensions,
    parser: async () => null,
    check: lang.check,
  })),
};

export default plugin;
