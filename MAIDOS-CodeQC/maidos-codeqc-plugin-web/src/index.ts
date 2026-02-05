/**
 * MAIDOS CodeQC Plugin: Web Languages
 * 
 * 支援語言：
 * - PHP (Laravel, WordPress, Drupal, Symfony)
 * - Ruby (Rails, Sinatra, Jekyll)
 */

export * from './types.js';
export { phpPlugin } from './languages/php.js';
export { rubyPlugin } from './languages/ruby.js';

import { phpPlugin } from './languages/php.js';
import { rubyPlugin } from './languages/ruby.js';
import type { LanguagePlugin } from './types.js';

export const plugins: LanguagePlugin[] = [
  phpPlugin,
  rubyPlugin,
];

export default plugins;
