/**
 * MAIDOS CodeQC Plugin: Scripting Languages
 * 
 * 支援語言：
 * - Shell/Bash (Bash, Sh, Zsh, Fish)
 * - PowerShell (Windows PowerShell, PowerShell Core)
 * - Perl (Perl 5, CGI, Mojolicious)
 * - Lua (Lua 5.x, LuaJIT, LÖVE, OpenResty)
 */

export * from './types.js';
export { shellPlugin } from './languages/shell.js';
export { powershellPlugin } from './languages/powershell.js';
export { perlPlugin } from './languages/perl.js';
export { luaPlugin } from './languages/lua.js';

import { shellPlugin } from './languages/shell.js';
import { powershellPlugin } from './languages/powershell.js';
import { perlPlugin } from './languages/perl.js';
import { luaPlugin } from './languages/lua.js';
import type { LanguagePlugin } from './types.js';

export const plugins: LanguagePlugin[] = [
  shellPlugin,
  powershellPlugin,
  perlPlugin,
  luaPlugin,
];

export default plugins;
