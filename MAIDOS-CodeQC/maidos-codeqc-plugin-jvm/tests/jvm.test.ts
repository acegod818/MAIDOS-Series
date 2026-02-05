/**
 * JVM Plugin Tests
 */

import { describe, it, expect } from 'vitest';
import { checkJava, checkKotlin, checkScala, checkGroovy, checkClojure, isJVMFile, checkJVMFile } from '../src/index.js';

describe('Java', () => {
  it('should detect hardcoded password', () => {
    const code = `String password = "secret123";`;
    const violations = checkJava(code, 'Config.java');
    expect(violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('should detect empty catch', () => {
    const code = `try { foo(); } catch (Exception e) { }`;
    const violations = checkJava(code, 'App.java');
    expect(violations.some(v => v.ruleId === 'R05')).toBe(true);
  });

  it('should detect unclosed resource', () => {
    const code = `FileInputStream fis = new FileInputStream("file.txt");`;
    const violations = checkJava(code, 'Reader.java');
    expect(violations.some(v => v.ruleId === 'R05')).toBe(true);
  });

  it('should detect unsafe deserialization', () => {
    const code = `ObjectInputStream ois = new ObjectInputStream(stream);`;
    const violations = checkJava(code, 'Deserializer.java');
    expect(violations.some(v => v.ruleId === 'R02')).toBe(true);
  });

  it('should detect public mutable static', () => {
    const code = `public static String config = "default";`;
    const violations = checkJava(code, 'Config.java');
    expect(violations.some(v => v.ruleId === 'P07')).toBe(true);
  });
});

describe('Kotlin', () => {
  it('should detect hardcoded password', () => {
    const code = `val password = "secret123"`;
    const violations = checkKotlin(code, 'Config.kt');
    expect(violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('should detect !! force unwrap', () => {
    const code = `val name = user!!.name`;
    const violations = checkKotlin(code, 'User.kt');
    expect(violations.some(v => v.ruleId === 'R05')).toBe(true);
  });

  it('should detect empty catch', () => {
    const code = `try { foo() } catch (e: Exception) { }`;
    const violations = checkKotlin(code, 'App.kt');
    expect(violations.some(v => v.ruleId === 'R05')).toBe(true);
  });

  it('should warn on excessive lateinit', () => {
    const code = Array(6).fill('lateinit var field: String').join('\n');
    const violations = checkKotlin(code, 'App.kt');
    expect(violations.some(v => v.ruleId === 'P07')).toBe(true);
  });
});

describe('Scala', () => {
  it('should detect hardcoded password', () => {
    const code = `val password = "secret123"`;
    const violations = checkScala(code, 'Config.scala');
    expect(violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('should detect null usage', () => {
    const code = `val name = null`;
    const violations = checkScala(code, 'App.scala');
    expect(violations.some(v => v.ruleId === 'R05')).toBe(true);
  });

  it('should warn on excessive var usage', () => {
    const code = Array(4).fill('var x = 1').join('\n');
    const violations = checkScala(code, 'App.scala');
    expect(violations.some(v => v.ruleId === 'P07')).toBe(true);
  });
});

describe('Groovy', () => {
  it('should detect hardcoded password', () => {
    const code = `def password = "secret123"`;
    const violations = checkGroovy(code, 'Config.groovy');
    expect(violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('should detect Gradle signing password', () => {
    const code = `signingPassword = "mypassword"`;
    const violations = checkGroovy(code, 'build.gradle');
    expect(violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('should detect GString SQL injection', () => {
    const code = `sql.execute("SELECT * FROM users WHERE id = \${userId}")`;
    const violations = checkGroovy(code, 'Dao.groovy');
    expect(violations.some(v => v.ruleId === 'R02')).toBe(true);
  });

  it('should detect Eval usage', () => {
    const code = `Eval.me("println 'hello'")`;
    const violations = checkGroovy(code, 'Script.groovy');
    expect(violations.some(v => v.ruleId === 'R02')).toBe(true);
  });
});

describe('Clojure', () => {
  it('should detect hardcoded password', () => {
    const code = `(def password "secret123")`;
    const violations = checkClojure(code, 'config.clj');
    expect(violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('should detect map with password key', () => {
    const code = `{:password "secret123"}`;
    const violations = checkClojure(code, 'config.clj');
    expect(violations.some(v => v.ruleId === 'R01')).toBe(true);
  });

  it('should detect empty catch', () => {
    const code = `(catch Exception _ )`;
    const violations = checkClojure(code, 'app.clj');
    expect(violations.some(v => v.ruleId === 'R05')).toBe(true);
  });

  it('should warn on excessive mutable state', () => {
    const code = Array(6).fill('(atom nil)').join('\n');
    const violations = checkClojure(code, 'app.clj');
    expect(violations.some(v => v.ruleId === 'P07')).toBe(true);
  });
});

describe('Plugin Interface', () => {
  it('should identify JVM files', () => {
    expect(isJVMFile('App.java')).toBe(true);
    expect(isJVMFile('App.kt')).toBe(true);
    expect(isJVMFile('App.scala')).toBe(true);
    expect(isJVMFile('build.gradle')).toBe(true);
    expect(isJVMFile('core.clj')).toBe(true);
    expect(isJVMFile('app.ts')).toBe(false);
    expect(isJVMFile('app.py')).toBe(false);
  });

  it('should check JVM files correctly', () => {
    const javaCode = `String password = "secret";`;
    const violations = checkJVMFile(javaCode, 'Config.java');
    expect(violations.length).toBeGreaterThan(0);
  });

  it('should return empty for non-JVM files', () => {
    const violations = checkJVMFile('const x = 1;', 'app.ts');
    expect(violations).toHaveLength(0);
  });
});
