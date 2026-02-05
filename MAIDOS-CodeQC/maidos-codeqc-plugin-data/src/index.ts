/**
 * MAIDOS CodeQC Plugin: Data/ML Languages
 * 
 * 支援語言：
 * - SQL (PostgreSQL, MySQL, SQLite, SQL Server, Oracle)
 * - R (RStudio, Shiny, tidyverse)
 * - Julia (Flux, Pluto, JuMP)
 */

export * from './types.js';
export { sqlPlugin } from './languages/sql.js';
export { rPlugin } from './languages/r.js';
export { juliaPlugin } from './languages/julia.js';

import { sqlPlugin } from './languages/sql.js';
import { rPlugin } from './languages/r.js';
import { juliaPlugin } from './languages/julia.js';
import type { LanguagePlugin } from './types.js';

export const plugins: LanguagePlugin[] = [
  sqlPlugin,
  rPlugin,
  juliaPlugin,
];

export default plugins;
