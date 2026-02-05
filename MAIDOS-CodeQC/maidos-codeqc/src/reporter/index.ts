/**
 * Reporters Module
 */

export { consoleReporter, default as console } from './console.js';
export { jsonReporter, default as json } from './json.js';
export { htmlReporter, default as html } from './html.js';

import type { Reporter } from '../types.js';
import { consoleReporter } from './console.js';
import { jsonReporter } from './json.js';
import { htmlReporter } from './html.js';

export const reporters: Record<string, Reporter> = {
  console: consoleReporter,
  json: jsonReporter,
  html: htmlReporter,
};

export function getReporter(name: string): Reporter {
  const reporter = reporters[name];
  if (!reporter) {
    throw new Error(`Unknown reporter: ${name}. Available: ${Object.keys(reporters).join(', ')}`);
  }
  return reporter;
}
