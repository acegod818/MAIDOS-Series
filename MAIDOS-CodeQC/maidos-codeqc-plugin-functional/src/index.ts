/**
 * MAIDOS CodeQC Plugin: Functional Languages
 * 
 * 支援語言：
 * - Elixir (Phoenix, Ecto, LiveView)
 * - Haskell (GHC, Yesod, Servant)
 * - OCaml (Reason, ReScript, Dune)
 * - Erlang (OTP, Cowboy, RabbitMQ)
 */

export * from './types.js';
export { elixirPlugin } from './languages/elixir.js';
export { haskellPlugin } from './languages/haskell.js';
export { ocamlPlugin } from './languages/ocaml.js';
export { erlangPlugin } from './languages/erlang.js';

import { elixirPlugin } from './languages/elixir.js';
import { haskellPlugin } from './languages/haskell.js';
import { ocamlPlugin } from './languages/ocaml.js';
import { erlangPlugin } from './languages/erlang.js';
import type { LanguagePlugin } from './types.js';

export const plugins: LanguagePlugin[] = [
  elixirPlugin,
  haskellPlugin,
  ocamlPlugin,
  erlangPlugin,
];

export default plugins;
