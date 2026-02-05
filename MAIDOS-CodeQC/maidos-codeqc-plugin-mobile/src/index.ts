/**
 * @maidos/codeqc-plugin-mobile
 * Mobile languages support: Swift, Dart, Objective-C
 */

import type { Violation } from '@maidos/codeqc';

export { SWIFT_CONFIG, checkSwift } from './languages/swift.js';
export { DART_CONFIG, checkDart } from './languages/dart.js';
export { OBJC_CONFIG, checkObjC } from './languages/objc.js';

import { SWIFT_CONFIG, checkSwift } from './languages/swift.js';
import { DART_CONFIG, checkDart } from './languages/dart.js';
import { OBJC_CONFIG, checkObjC } from './languages/objc.js';

export interface MobileLanguageSupport {
  id: string;
  name: string;
  extensions: string[];
  check: (source: string, file: string) => Violation[];
}

export const LANGUAGES: MobileLanguageSupport[] = [
  { ...SWIFT_CONFIG, check: checkSwift },
  { ...DART_CONFIG, check: checkDart },
  { ...OBJC_CONFIG, check: checkObjC },
];

export function getLanguageByExtension(file: string): MobileLanguageSupport | undefined {
  const ext = '.' + file.split('.').pop()?.toLowerCase();
  return LANGUAGES.find(lang => lang.extensions.includes(ext));
}

export function checkMobileFile(source: string, file: string): Violation[] {
  const lang = getLanguageByExtension(file);
  return lang ? lang.check(source, file) : [];
}

export function isMobileFile(file: string): boolean {
  return getLanguageByExtension(file) !== undefined;
}

export const plugin = {
  name: '@maidos/codeqc-plugin-mobile',
  version: '0.1.0',
  languages: LANGUAGES.map(lang => ({
    id: lang.id,
    extensions: lang.extensions,
    parser: async () => null,
    check: lang.check,
  })),
};

export default plugin;
