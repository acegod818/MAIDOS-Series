/**
 * MAIDOS CodeQC - Checker Behavior Verification
 * 
 * 驗證每個 checker 的實際檢測能力
 * 確保沒有「假實作」
 */

import { describe, it, expect } from 'vitest';
import { checkRedlines } from '../src/rules/b-redlines.js';
import { checkProhibitions } from '../src/rules/b-prohibitions.js';

describe('紅線 Checker 行為驗證', () => {
  describe('R01 硬編碼憑證', () => {
    it('檢測硬編碼密碼', () => {
      const v = checkRedlines('const password = "secret123"', 'test.ts');
      expect(v.some(x => x.ruleId === 'R01')).toBe(true);
    });
    
    it('檢測 API Key', () => {
      const v = checkRedlines('const api_key = "sk-abc123456789"', 'test.ts');
      expect(v.some(x => x.ruleId === 'R01')).toBe(true);
    });
    
    it('不報告環境變數', () => {
      const v = checkRedlines('const password = process.env.PASSWORD', 'test.ts');
      expect(v.some(x => x.ruleId === 'R01')).toBe(false);
    });
  });
  
  describe('R02 跳過安全檢查', () => {
    it('檢測 SQL 注入', () => {
      const v = checkRedlines('db.query(`SELECT * FROM users WHERE id = ${id}`)', 'test.ts');
      expect(v.some(x => x.ruleId === 'R02')).toBe(true);
    });
    
    it('檢測 eval', () => {
      const v = checkRedlines('eval(userInput)', 'test.ts');
      expect(v.some(x => x.ruleId === 'R02')).toBe(true);
    });
    
    it('檢測不安全反序列化', () => {
      const v = checkRedlines('pickle.load(data)', 'test.py');
      expect(v.some(x => x.ruleId === 'R02')).toBe(true);
    });
  });
  
  describe('R03 刪除審計日誌', () => {
    it('檢測 rm 日誌', () => {
      const v = checkRedlines('rm -rf /var/log/audit', 'test.sh');
      expect(v.some(x => x.ruleId === 'R03')).toBe(true);
    });
    
    it('檢測 SQL DELETE 審計', () => {
      const v = checkRedlines('DELETE FROM audit_log WHERE date < now()', 'test.sql');
      expect(v.some(x => x.ruleId === 'R03')).toBe(true);
    });
  });
  
  describe('R05 忽略錯誤處理', () => {
    it('檢測空 catch (TS)', () => {
      const v = checkRedlines('try { x() } catch (e) { }', 'test.ts');
      expect(v.some(x => x.ruleId === 'R05')).toBe(true);
    });
    
    it('檢測 except pass (Python)', () => {
      const v = checkRedlines('except: pass', 'test.py');
      expect(v.some(x => x.ruleId === 'R05')).toBe(true);
    });
    
    it('檢測 unwrap (Rust)', () => {
      const v = checkRedlines('result.unwrap()', 'test.rs');
      expect(v.some(x => x.ruleId === 'R05')).toBe(true);
    });
  });
  
  describe('R07 關閉安全功能', () => {
    it('檢測關閉 SSL 驗證', () => {
      const v = checkRedlines('rejectUnauthorized: false', 'test.ts');
      expect(v.some(x => x.ruleId === 'R07')).toBe(true);
    });
    
    it('檢測 NODE_TLS_REJECT_UNAUTHORIZED', () => {
      const v = checkRedlines('NODE_TLS_REJECT_UNAUTHORIZED=0', 'test.ts');
      expect(v.some(x => x.ruleId === 'R07')).toBe(true);
    });
  });
  
  describe('R08 使用已知漏洞', () => {
    it('檢測 strcpy', () => {
      const v = checkRedlines('strcpy(dest, src);', 'test.c');
      expect(v.some(x => x.ruleId === 'R08')).toBe(true);
    });
    
    it('檢測 gets', () => {
      const v = checkRedlines('gets(buffer);', 'test.c');
      expect(v.some(x => x.ruleId === 'R08')).toBe(true);
    });
    
    it('檢測 MD5', () => {
      const v = checkRedlines('MD5(password)', 'test.ts');
      expect(v.some(x => x.ruleId === 'R08')).toBe(true);
    });
  });
  
  describe('R09 無限制資源', () => {
    it('檢測 while(true)', () => {
      const v = checkRedlines('while (true) { process(); }', 'test.ts');
      expect(v.some(x => x.ruleId === 'R09')).toBe(true);
    });
    
    it('檢測無 LIMIT 查詢', () => {
      const v = checkRedlines('SELECT * FROM users', 'test.ts');
      expect(v.some(x => x.ruleId === 'R09')).toBe(true);
    });
  });
  
  describe('R10 明文傳輸敏感', () => {
    it('檢測 HTTP 登入', () => {
      const v = checkRedlines('"http://api.example.com/login"', 'test.ts');
      expect(v.some(x => x.ruleId === 'R10')).toBe(true);
    });
    
    it('檢測 FTP', () => {
      const v = checkRedlines('"ftp://server.com/data"', 'test.ts');
      expect(v.some(x => x.ruleId === 'R10')).toBe(true);
    });
    
    it('不報告 HTTPS', () => {
      const v = checkRedlines('"https://api.example.com/login"', 'test.ts');
      expect(v.some(x => x.ruleId === 'R10')).toBe(false);
    });
  });
  
  describe('R12 偽造測試結果', () => {
    it('檢測 skip 測試', () => {
      const v = checkRedlines('it.skip("test", () => {})', 'test.spec.ts');
      expect(v.some(x => x.ruleId === 'R12')).toBe(true);
    });
    
    it('檢測 assert(true)', () => {
      const v = checkRedlines('assert(true)', 'test.spec.ts');
      expect(v.some(x => x.ruleId === 'R12')).toBe(true);
    });
    
    it('不報告非測試檔案', () => {
      const v = checkRedlines('assert(true)', 'main.ts');
      expect(v.some(x => x.ruleId === 'R12')).toBe(false);
    });
  });
});

describe('禁止 Checker 行為驗證', () => {
  describe('P03 複製粘貼', () => {
    it('檢測多組重複代碼', () => {
      const code = [
        ...Array(4).fill('const a = func1();'),
        ...Array(4).fill('const b = func2();'),
        ...Array(4).fill('const c = func3();'),
        ...Array(4).fill('const d = func4();'),
      ].join('\n');
      const v = checkProhibitions(code, 'test.ts');
      expect(v.some(x => x.ruleId === 'P03')).toBe(true);
    });
  });
  
  describe('P04 魔法數字', () => {
    it('檢測運算中的魔法數字', () => {
      const v = checkProhibitions('const seconds = hours * 3600;', 'main.ts');
      expect(v.some(x => x.ruleId === 'P04')).toBe(true);
    });
    
    it('不報告純常量定義', () => {
      const v = checkProhibitions('const SECONDS_PER_HOUR = 3600;', 'main.ts');
      expect(v.some(x => x.ruleId === 'P04')).toBe(false);
    });
    
    it('不報告測試檔案', () => {
      const v = checkProhibitions('expect(result).toBe(3600);', 'test.spec.ts');
      expect(v.some(x => x.ruleId === 'P04')).toBe(false);
    });
  });
  
  describe('P05 超長函數', () => {
    it('檢測超過 50 行函數', () => {
      const code = 'function test() {\n' + Array(55).fill('  x++;').join('\n') + '\n}';
      const v = checkProhibitions(code, 'test.ts');
      expect(v.some(x => x.ruleId === 'P05')).toBe(true);
    });
    
    it('不報告 50 行以內', () => {
      const code = 'function test() {\n' + Array(45).fill('  x++;').join('\n') + '\n}';
      const v = checkProhibitions(code, 'test.ts');
      expect(v.some(x => x.ruleId === 'P05')).toBe(false);
    });
  });
  
  describe('P06 深層嵌套', () => {
    it('檢測超過 3 層嵌套', () => {
      const v = checkProhibitions('if(a){if(b){if(c){if(d){}}}}', 'test.ts');
      expect(v.some(x => x.ruleId === 'P06')).toBe(true);
    });
  });
  
  describe('P07 全局狀態', () => {
    it('檢測 window 全局', () => {
      const v = checkProhibitions('window.myGlobal = 123;', 'test.ts');
      expect(v.some(x => x.ruleId === 'P07')).toBe(true);
    });
    
    it('檢測 global 全局', () => {
      const v = checkProhibitions('global.config = {};', 'test.ts');
      expect(v.some(x => x.ruleId === 'P07')).toBe(true);
    });
  });
  
  describe('P09 無意義命名', () => {
    it('檢測 temp', () => {
      const v = checkProhibitions('const temp = getData();', 'main.ts');
      expect(v.some(x => x.ruleId === 'P09')).toBe(true);
    });
    
    it('檢測 data', () => {
      const v = checkProhibitions('let data = fetch();', 'main.ts');
      expect(v.some(x => x.ruleId === 'P09')).toBe(true);
    });
    
    it('不報告測試檔案', () => {
      const v = checkProhibitions('const temp = getData();', 'test.spec.ts');
      expect(v.some(x => x.ruleId === 'P09')).toBe(false);
    });
  });
  
  describe('P10 過長參數', () => {
    it('檢測超過 5 個參數', () => {
      const v = checkProhibitions('function test(a, b, c, d, e, f, g) {}', 'test.ts');
      expect(v.some(x => x.ruleId === 'P10')).toBe(true);
    });
    
    it('不報告 5 個以內', () => {
      const v = checkProhibitions('function test(a, b, c, d, e) {}', 'test.ts');
      expect(v.some(x => x.ruleId === 'P10')).toBe(false);
    });
  });
  
  describe('P12 註釋代碼', () => {
    it('檢測大量註釋代碼', () => {
      const code = Array(15).fill('// const x = 1;').join('\n');
      const v = checkProhibitions(code, 'test.ts');
      expect(v.some(x => x.ruleId === 'P12')).toBe(true);
    });
  });
  
  describe('P13 TODO 堆積', () => {
    it('檢測超過 10 個 TODO', () => {
      const code = Array(15).fill('// TODO: fix this').join('\n');
      const v = checkProhibitions(code, 'test.ts');
      expect(v.some(x => x.ruleId === 'P13')).toBe(true);
    });
    
    it('不報告 10 個以內', () => {
      const code = Array(8).fill('// TODO: fix').join('\n');
      const v = checkProhibitions(code, 'test.ts');
      expect(v.some(x => x.ruleId === 'P13')).toBe(false);
    });
  });
  
  describe('P14 依賴膨脹', () => {
    it('檢測超過 50 個依賴', () => {
      const pkg = { dependencies: Object.fromEntries(Array(55).fill(0).map((_, i) => ['dep' + i, '1.0.0'])) };
      const v = checkProhibitions(JSON.stringify(pkg), 'package.json');
      expect(v.some(x => x.ruleId === 'P14')).toBe(true);
    });
  });
});
