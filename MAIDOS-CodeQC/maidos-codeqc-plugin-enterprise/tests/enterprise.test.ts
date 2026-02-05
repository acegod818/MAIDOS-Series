/**
 * Enterprise Plugin Behavior Tests
 * 
 * 驗證昭和遺產語言 checker 實際行為
 */

import { describe, it, expect } from 'vitest';
import { cobolPlugin } from '../src/languages/cobol.js';
import { abapPlugin } from '../src/languages/abap.js';
import { plsqlPlugin } from '../src/languages/plsql.js';
import { fortranPlugin } from '../src/languages/fortran.js';
import { vbaPlugin } from '../src/languages/vba.js';
import { rpgPlugin } from '../src/languages/rpg.js';

// =============================================================================
// COBOL Tests
// =============================================================================

describe('COBOL checker behavior', () => {
  it('detects hardcoded credentials (R01)', () => {
    const code = `MOVE "secret123" TO WS-PASSWORD.`;
    const violations = cobolPlugin.checkSource(code, 'test.cob');
    expect(violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('detects SQL injection risk (R02)', () => {
    const code = `EXEC SQL EXECUTE IMMEDIATE :WS-QUERY END-EXEC.`;
    const violations = cobolPlugin.checkSource(code, 'test.cob');
    expect(violations.some(v => v.ruleId === 'R02')).toBe(true);
  });

  it('detects infinite loop (R09)', () => {
    const code = `PERFORM PROCESS-DATA UNTIL 1 = 0.`;
    const violations = cobolPlugin.checkSource(code, 'test.cob');
    expect(violations.some(v => v.ruleId === 'R09')).toBe(true);
  });

  it('passes clean code', () => {
    const code = `MOVE WS-INPUT TO WS-OUTPUT.`;
    const violations = cobolPlugin.checkSource(code, 'test.cob');
    expect(violations.length).toBe(0);
  });
});

// =============================================================================
// ABAP Tests
// =============================================================================

describe('ABAP checker behavior', () => {
  it('detects hardcoded credentials (R01)', () => {
    const code = `lv_password = 'secret123'.`;
    const violations = abapPlugin.checkSource(code, 'test.abap');
    expect(violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('detects bypassed authority check (R07)', () => {
    const code = `AUTHORITY-CHECK OBJECT 'S_TCODE' DUMMY.`;
    const violations = abapPlugin.checkSource(code, 'test.abap');
    expect(violations.some(v => v.ruleId === 'R07')).toBe(true);
  });

  it('detects unlimited SELECT (R09)', () => {
    const code = `SELECT * FROM mara INTO TABLE lt_mara.`;
    const violations = abapPlugin.checkSource(code, 'test.abap');
    expect(violations.some(v => v.ruleId === 'R09')).toBe(true);
  });

  it('passes clean code', () => {
    const code = `SELECT * FROM mara INTO TABLE lt_mara UP TO 100 ROWS WHERE matnr = lv_matnr.`;
    const violations = abapPlugin.checkSource(code, 'test.abap');
    expect(violations.filter(v => v.ruleId === 'R09').length).toBe(0);
  });
});

// =============================================================================
// PL/SQL Tests
// =============================================================================

describe('PL/SQL checker behavior', () => {
  it('detects hardcoded credentials (R01)', () => {
    const code = `v_password := 'secret123';`;
    const violations = plsqlPlugin.checkSource(code, 'test.pls');
    expect(violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('detects dynamic SQL injection (R02)', () => {
    const code = `EXECUTE IMMEDIATE v_sql || v_user_input;`;
    const violations = plsqlPlugin.checkSource(code, 'test.pls');
    expect(violations.some(v => v.ruleId === 'R02')).toBe(true);
  });

  it('detects dangerous privilege grant (R07)', () => {
    const code = `GRANT DBA TO scott;`;
    const violations = plsqlPlugin.checkSource(code, 'test.pls');
    expect(violations.some(v => v.ruleId === 'R07')).toBe(true);
  });

  it('passes clean code', () => {
    const code = `SELECT employee_name INTO v_name FROM employees WHERE id = p_id;`;
    const violations = plsqlPlugin.checkSource(code, 'test.pls');
    expect(violations.length).toBe(0);
  });
});

// =============================================================================
// Fortran Tests
// =============================================================================

describe('Fortran checker behavior', () => {
  it('detects hardcoded credentials (R01)', () => {
    const code = `password = "secret123"`;
    const violations = fortranPlugin.checkSource(code, 'test.f90');
    expect(violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('detects system command (R02)', () => {
    const code = `CALL SYSTEM('rm -rf /tmp')`;
    const violations = fortranPlugin.checkSource(code, 'test.f90');
    expect(violations.some(v => v.ruleId === 'R02')).toBe(true);
  });

  it('detects COMMON block (P07)', () => {
    const code = `COMMON /SHARED/ x, y, z`;
    const violations = fortranPlugin.checkSource(code, 'test.f90');
    expect(violations.some(v => v.ruleId === 'P07')).toBe(true);
  });

  it('passes clean code', () => {
    const code = `result = a + b * c`;
    const violations = fortranPlugin.checkSource(code, 'test.f90');
    expect(violations.length).toBe(0);
  });
});

// =============================================================================
// VBA Tests
// =============================================================================

describe('VBA checker behavior', () => {
  it('detects hardcoded credentials (R01)', () => {
    const code = `strPassword = "secret123"`;
    const violations = vbaPlugin.checkSource(code, 'test.bas');
    expect(violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('detects Shell command (R02)', () => {
    const code = `Shell("cmd.exe /c dir")`;
    const violations = vbaPlugin.checkSource(code, 'test.bas');
    expect(violations.some(v => v.ruleId === 'R02')).toBe(true);
  });

  it('detects On Error Resume Next (R05)', () => {
    const code = `On Error Resume Next`;
    const violations = vbaPlugin.checkSource(code, 'test.bas');
    expect(violations.some(v => v.ruleId === 'R05')).toBe(true);
  });

  it('passes clean code', () => {
    const code = `MsgBox "Hello World"`;
    const violations = vbaPlugin.checkSource(code, 'test.bas');
    expect(violations.length).toBe(0);
  });
});

// =============================================================================
// RPG Tests
// =============================================================================

describe('RPG checker behavior', () => {
  it('detects hardcoded credentials (R01)', () => {
    const code = `DCL-C PASSWORD 'secret123';`;
    const violations = rpgPlugin.checkSource(code, 'test.rpgle');
    expect(violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('detects QCMDEXC (R02)', () => {
    const code = `QCMDEXC(cmd:len);`;
    const violations = rpgPlugin.checkSource(code, 'test.rpgle');
    expect(violations.some(v => v.ruleId === 'R02')).toBe(true);
  });

  it('detects infinite loop (R09)', () => {
    const code = `DOW *ON;`;
    const violations = rpgPlugin.checkSource(code, 'test.rpgle');
    expect(violations.some(v => v.ruleId === 'R09')).toBe(true);
  });

  it('passes clean code', () => {
    const code = `result = a + b;`;
    const violations = rpgPlugin.checkSource(code, 'test.rpgle');
    expect(violations.length).toBe(0);
  });
});
