/**
 * @maidos/codeqc-plugin-jvm
 * JVM languages support for MAIDOS CodeQC
 * 
 * Supported languages:
 * - Java (.java)
 * - Kotlin (.kt, .kts)
 * - Scala (.scala, .sc)
 * - Groovy (.groovy, .gradle)
 * - Clojure (.clj, .cljs, .cljc)
 */

import type { Violation } from '@maidos/codeqc';

// Language modules
export { JAVA_CONFIG, checkJava, checkJavaRedlines, checkJavaProhibitions } from './languages/java.js';
export { KOTLIN_CONFIG, checkKotlin, checkKotlinRedlines, checkKotlinProhibitions } from './languages/kotlin.js';
export { SCALA_CONFIG, checkScala, checkScalaRedlines, checkScalaProhibitions } from './languages/scala.js';
export { GROOVY_CONFIG, checkGroovy, checkGroovyRedlines, checkGroovyProhibitions } from './languages/groovy.js';
export { CLOJURE_CONFIG, checkClojure, checkClojureRedlines, checkClojureProhibitions } from './languages/clojure.js';

import { JAVA_CONFIG, checkJava } from './languages/java.js';
import { KOTLIN_CONFIG, checkKotlin } from './languages/kotlin.js';
import { SCALA_CONFIG, checkScala } from './languages/scala.js';
import { GROOVY_CONFIG, checkGroovy } from './languages/groovy.js';
import { CLOJURE_CONFIG, checkClojure } from './languages/clojure.js';

// =============================================================================
// Plugin Definition
// =============================================================================

export interface JVMLanguageSupport {
  id: string;
  name: string;
  extensions: string[];
  check: (source: string, file: string) => Violation[];
}

export const LANGUAGES: JVMLanguageSupport[] = [
  { ...JAVA_CONFIG, check: checkJava },
  { ...KOTLIN_CONFIG, check: checkKotlin },
  { ...SCALA_CONFIG, check: checkScala },
  { ...GROOVY_CONFIG, check: checkGroovy },
  { ...CLOJURE_CONFIG, check: checkClojure },
];

/**
 * 根據檔案副檔名取得對應的語言支援
 */
export function getLanguageByExtension(file: string): JVMLanguageSupport | undefined {
  const ext = '.' + file.split('.').pop()?.toLowerCase();
  return LANGUAGES.find(lang => lang.extensions.includes(ext));
}

/**
 * 檢查 JVM 語言檔案
 */
export function checkJVMFile(source: string, file: string): Violation[] {
  const lang = getLanguageByExtension(file);
  if (!lang) {
    return [];
  }
  return lang.check(source, file);
}

/**
 * 判斷是否支援該檔案
 */
export function isJVMFile(file: string): boolean {
  return getLanguageByExtension(file) !== undefined;
}

// =============================================================================
// Plugin Export (for @maidos/codeqc plugin system)
// =============================================================================

export const plugin = {
  name: '@maidos/codeqc-plugin-jvm',
  version: '0.1.0',
  languages: LANGUAGES.map(lang => ({
    id: lang.id,
    extensions: lang.extensions,
    parser: async () => null, // Regex-based, no parser needed
    check: lang.check,
  })),
};

export default plugin;
