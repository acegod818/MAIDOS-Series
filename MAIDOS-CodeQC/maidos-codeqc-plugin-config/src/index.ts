/**
 * MAIDOS CodeQC Plugin: Configuration Languages
 * 
 * 支援語言：
 * - YAML (Docker Compose, Kubernetes, Ansible, CI/CD)
 * - JSON (package.json, tsconfig, eslintrc)
 * - TOML (Cargo.toml, pyproject.toml)
 * - XML (pom.xml, AndroidManifest, .csproj)
 */

export * from './types.js';
export { yamlPlugin } from './languages/yaml.js';
export { jsonPlugin } from './languages/json.js';
export { tomlPlugin } from './languages/toml.js';
export { xmlPlugin } from './languages/xml.js';

import { yamlPlugin } from './languages/yaml.js';
import { jsonPlugin } from './languages/json.js';
import { tomlPlugin } from './languages/toml.js';
import { xmlPlugin } from './languages/xml.js';
import type { LanguagePlugin } from './types.js';

export const plugins: LanguagePlugin[] = [
  yamlPlugin,
  jsonPlugin,
  tomlPlugin,
  xmlPlugin,
];

export default plugins;
