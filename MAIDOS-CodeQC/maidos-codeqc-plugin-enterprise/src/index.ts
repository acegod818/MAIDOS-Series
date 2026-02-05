/**
 * MAIDOS CodeQC Plugin: Enterprise Legacy Languages (昭和遺產)
 * 
 * 支援語言：
 * - COBOL (金融、保險、政府、銀行核心系統)
 * - ABAP (SAP ERP, S/4HANA)
 * - PL/SQL (Oracle Database)
 * - Fortran (科學計算、HPC、航太)
 * - VBA (Excel, Access, Office 自動化)
 * - RPG (IBM i/AS/400, 銀行、製造業)
 */

export * from './types.js';
export { cobolPlugin } from './languages/cobol.js';
export { abapPlugin } from './languages/abap.js';
export { plsqlPlugin } from './languages/plsql.js';
export { fortranPlugin } from './languages/fortran.js';
export { vbaPlugin } from './languages/vba.js';
export { rpgPlugin } from './languages/rpg.js';

import { cobolPlugin } from './languages/cobol.js';
import { abapPlugin } from './languages/abap.js';
import { plsqlPlugin } from './languages/plsql.js';
import { fortranPlugin } from './languages/fortran.js';
import { vbaPlugin } from './languages/vba.js';
import { rpgPlugin } from './languages/rpg.js';
import type { LanguagePlugin } from './types.js';

export const plugins: LanguagePlugin[] = [
  cobolPlugin,
  abapPlugin,
  plsqlPlugin,
  fortranPlugin,
  vbaPlugin,
  rpgPlugin,
];

export default plugins;
