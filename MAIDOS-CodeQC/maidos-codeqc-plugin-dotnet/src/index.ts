/**
 * @maidos/codeqc-plugin-dotnet
 * .NET languages support for MAIDOS CodeQC
 */

import type { Violation } from '@maidos/codeqc';

export { CSHARP_CONFIG, checkCSharp } from './languages/csharp.js';
export { FSHARP_CONFIG, checkFSharp } from './languages/fsharp.js';
export { VBNET_CONFIG, checkVBNet } from './languages/vbnet.js';

import { CSHARP_CONFIG, checkCSharp } from './languages/csharp.js';
import { FSHARP_CONFIG, checkFSharp } from './languages/fsharp.js';
import { VBNET_CONFIG, checkVBNet } from './languages/vbnet.js';

export interface DotNetLanguageSupport {
  id: string;
  name: string;
  extensions: string[];
  check: (source: string, file: string) => Violation[];
}

export const LANGUAGES: DotNetLanguageSupport[] = [
  { ...CSHARP_CONFIG, check: checkCSharp },
  { ...FSHARP_CONFIG, check: checkFSharp },
  { ...VBNET_CONFIG, check: checkVBNet },
];

export function getLanguageByExtension(file: string): DotNetLanguageSupport | undefined {
  const ext = '.' + file.split('.').pop()?.toLowerCase();
  return LANGUAGES.find(lang => lang.extensions.includes(ext));
}

export function checkDotNetFile(source: string, file: string): Violation[] {
  const lang = getLanguageByExtension(file);
  if (!lang) return [];
  return lang.check(source, file);
}

export function isDotNetFile(file: string): boolean {
  return getLanguageByExtension(file) !== undefined;
}

export const plugin = {
  name: '@maidos/codeqc-plugin-dotnet',
  version: '0.1.0',
  languages: LANGUAGES.map(lang => ({
    id: lang.id,
    extensions: lang.extensions,
    parser: async () => null,
    check: lang.check,
  })),
};

export default plugin;
