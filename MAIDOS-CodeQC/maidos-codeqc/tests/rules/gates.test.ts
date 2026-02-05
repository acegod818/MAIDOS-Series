/**
 * Gates & Dual-Axis Score Tests
 */

import { describe, it, expect } from 'vitest';
import {
  evaluateGate,
  createGateStatus,
  calculateComplianceScore,
  calculateOutcomeScore,
  calculateDualAxisScore,
  extractHandoverTags,
} from '../../src/rules/c-gates.js';

describe('Gate Evaluation', () => {
  describe('Gate-In', () => {
    it('should pass when all required items pass', () => {
      const result = evaluateGate('Gate-In', {
        '需求明確': true,
        '驗收標準定義': true,
        '技術選型確定': true,
        '依賴確認可用': true,
        '資源已評估': true,
      });
      expect(result.passed).toBe(true);
    });

    it('should fail when any required item fails', () => {
      const result = evaluateGate('Gate-In', {
        '需求明確': true,
        '驗收標準定義': false,
        '技術選型確定': true,
        '依賴確認可用': true,
        '資源已評估': true,
      });
      expect(result.passed).toBe(false);
    });
  });

  describe('Gate-Out', () => {
    it('should pass when all items pass', () => {
      const result = evaluateGate('Gate-Out', {
        '所有功能完成': true,
        '所有測試通過': true,
        '覆蓋率 ≥ 80%': true,
        '無紅線違規': true,
        '文檔完成': true,
        '代碼審查通過': true,
        '安全掃描通過': true,
      });
      expect(result.passed).toBe(true);
    });

    it('should fail when redline violation exists', () => {
      const result = evaluateGate('Gate-Out', {
        '所有功能完成': true,
        '所有測試通過': true,
        '覆蓋率 ≥ 80%': true,
        '無紅線違規': false,
        '文檔完成': true,
        '代碼審查通過': true,
        '安全掃描通過': true,
      });
      expect(result.passed).toBe(false);
    });
  });

  describe('createGateStatus', () => {
    it('should create complete gate status', () => {
      const status = createGateStatus(
        { '需求明確': true, '驗收標準定義': true, '技術選型確定': true, '依賴確認可用': true, '資源已評估': true },
        { '進度在 ±20% 內': true, '核心架構穩定': true, '核心邏輯有測試': true, '阻塞項有方案': true },
        { '所有功能完成': true, '所有測試通過': true, '覆蓋率 ≥ 80%': true, '無紅線違規': true, '文檔完成': true, '代碼審查通過': true, '安全掃描通過': true },
        { '用戶驗收測試通過': true, '性能指標達標': true, '可部署到目標環境': true, '回滾方案就緒': true, '監控告警配置': true }
      );
      
      expect(status.gateIn.passed).toBe(true);
      expect(status.gateMid.passed).toBe(true);
      expect(status.gateOut.passed).toBe(true);
      expect(status.gateAccept.passed).toBe(true);
    });
  });
});

describe('Dual-Axis Scoring', () => {
  describe('Compliance Score (X-Axis)', () => {
    it('should calculate weighted total correctly', () => {
      const score = calculateComplianceScore({
        codeStandard: 80,    // 15%
        architecture: 90,    // 20%
        security: 85,        // 25%
        testing: 80,         // 20%
        documentation: 70,   // 10%
        process: 75,         // 10%
      });
      
      // 80*0.15 + 90*0.20 + 85*0.25 + 80*0.20 + 70*0.10 + 75*0.10
      // = 12 + 18 + 21.25 + 16 + 7 + 7.5 = 81.75
      expect(score.total).toBe(81.75);
    });
  });

  describe('Outcome Score (Y-Axis)', () => {
    it('should calculate weighted total correctly', () => {
      const score = calculateOutcomeScore({
        functionality: 90,   // 30%
        quality: 85,         // 20%
        performance: 80,     // 20%
        usability: 75,       // 15%
        satisfaction: 80,    // 15%
      });
      
      // 90*0.30 + 85*0.20 + 80*0.20 + 75*0.15 + 80*0.15
      // = 27 + 17 + 16 + 11.25 + 12 = 83.25
      expect(score.total).toBe(83.25);
    });
  });

  describe('Grade Calculation', () => {
    it('should return grade A when both axes >= 80%', () => {
      const score = calculateDualAxisScore(
        { codeStandard: 80, architecture: 80, security: 80, testing: 80, documentation: 80, process: 80 },
        { functionality: 80, quality: 80, performance: 80, usability: 80, satisfaction: 80 }
      );
      expect(score.grade).toBe('A');
    });

    it('should return grade B when X < 80% and Y >= 80%', () => {
      const score = calculateDualAxisScore(
        { codeStandard: 70, architecture: 70, security: 70, testing: 70, documentation: 70, process: 70 },
        { functionality: 90, quality: 90, performance: 90, usability: 90, satisfaction: 90 }
      );
      expect(score.grade).toBe('B');
    });

    it('should return grade C when X >= 80% and Y < 80%', () => {
      const score = calculateDualAxisScore(
        { codeStandard: 90, architecture: 90, security: 90, testing: 90, documentation: 90, process: 90 },
        { functionality: 70, quality: 70, performance: 70, usability: 70, satisfaction: 70 }
      );
      expect(score.grade).toBe('C');
    });

    it('should return grade D when both axes < 80%', () => {
      const score = calculateDualAxisScore(
        { codeStandard: 60, architecture: 60, security: 60, testing: 60, documentation: 60, process: 60 },
        { functionality: 60, quality: 60, performance: 60, usability: 60, satisfaction: 60 }
      );
      expect(score.grade).toBe('D');
    });
  });
});

describe('Handover Tags', () => {
  it('should extract TODO tags', () => {
    const code = `
// @HANDOVER: 交接點說明
function foo() {}
// @WIP: 剩餘工作
// @BLOCKED: 等待 API
`;
    const tags = extractHandoverTags(code);
    
    expect(tags).toHaveLength(3);
    expect(tags[0]).toEqual({ tag: 'HANDOVER', line: 2, content: '交接點說明' });
    expect(tags[1]).toEqual({ tag: 'WIP', line: 4, content: '剩餘工作' });
    expect(tags[2]).toEqual({ tag: 'BLOCKED', line: 5, content: '等待 API' });
  });

  it('should extract all supported tags', () => {
    const code = `
// @DECISION: 選項 A 或 B
// @REVIEW: 需要審查邏輯
// @FIXME: 修復記憶體洩漏
// @HACK: 臨時方案
`;
    const tags = extractHandoverTags(code);
    
    expect(tags).toHaveLength(4);
    expect(tags.map(t => t.tag)).toEqual(['DECISION', 'REVIEW', 'FIXME', 'HACK']);
  });

  it('should return empty array when no tags', () => {
    const code = `function foo() { return 1; }`;
    const tags = extractHandoverTags(code);
    expect(tags).toHaveLength(0);
  });
});
