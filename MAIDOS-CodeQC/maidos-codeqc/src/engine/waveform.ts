/**
 * CodeQC v3.3 Engine — Waveform (三通道示波器)
 * 
 * CH1 (Y軸) 功能波形: 規格映射+測試+覆蓋+補完
 * CH2 (X軸) 品質波形: 編譯+Lint+紅線+安全
 * CH3 (Z軸) 真實波形: 詐欺掃描+IAV+BLDS+追溯
 * 
 * 每個通道輸出: amplitude (0-100) + status (PASS/FAIL/WARN)
 */

// =============================================================================
// Types
// =============================================================================

export type WaveformStatus = 'PASS' | 'FAIL' | 'WARN' | 'SKIP';

export interface ChannelReading {
  id: string;
  name: string;
  amplitude: number;       // 0-100
  status: WaveformStatus;
  evidence?: string;       // evidence/ 檔案路徑
  detail?: string;
}

export interface WaveformChannel {
  channel: 'CH1_Y' | 'CH2_X' | 'CH3_Z';
  name: string;
  readings: ChannelReading[];
  overall: WaveformStatus;
  score: number;           // 0-100 加權平均
}

export interface WaveformReport {
  timestamp: string;
  channels: [WaveformChannel, WaveformChannel, WaveformChannel];
  allPass: boolean;
  compositeScore: number;  // 三通道加權
}

// =============================================================================
// Channel Builders
// =============================================================================

/** CH1 (Y軸) 功能波形 */
export function buildYChannel(input: {
  specMapped: boolean;          // Y1: SPEC↔impl 映射完整
  specMissingCount: number;
  testsPass: boolean;           // Y2: 測試全過
  testsFailed: number;
  testsTotal: number;
  coveragePercent: number;      // Y3: 覆蓋率
  coverageThreshold: number;
  implComplete: boolean;        // Y4: 補完 (mapping.log 無 MISSING)
}): WaveformChannel {
  const readings: ChannelReading[] = [
    {
      id: 'Y1', name: '規格有對應',
      amplitude: input.specMapped ? 100 : Math.max(0, 100 - input.specMissingCount * 20),
      status: input.specMapped ? 'PASS' : 'FAIL',
      evidence: 'mapping.log',
    },
    {
      id: 'Y2', name: '測試有過',
      amplitude: input.testsTotal > 0 ? ((input.testsTotal - input.testsFailed) / input.testsTotal) * 100 : 0,
      status: input.testsPass ? 'PASS' : 'FAIL',
      evidence: 'test.log',
    },
    {
      id: 'Y3', name: '測了多少',
      amplitude: input.coveragePercent,
      status: input.coveragePercent >= input.coverageThreshold ? 'PASS' : 'WARN',
      evidence: 'coverage.log',
    },
    {
      id: 'Y4', name: '功能完整',
      amplitude: input.implComplete ? 100 : 50,
      status: input.implComplete ? 'PASS' : 'WARN',
      evidence: 'impl.log',
    },
  ];
  return finalizeChannel('CH1_Y', '功能完整度', readings);
}

/** CH2 (X軸) 品質波形 */
export function buildXChannel(input: {
  buildErrors: number;          // X1: 編譯
  buildWarnings: number;
  lintErrors: number;           // X2: Lint
  lintWarnings: number;
  redlineViolations: number;    // X3: 紅線
  securityCritical: number;     // X4: 安全
  securityHigh: number;
}): WaveformChannel {
  const readings: ChannelReading[] = [
    {
      id: 'X1', name: '編譯乾淨',
      amplitude: input.buildErrors === 0 ? (input.buildWarnings === 0 ? 100 : 80) : 0,
      status: input.buildErrors === 0 ? (input.buildWarnings === 0 ? 'PASS' : 'WARN') : 'FAIL',
      evidence: 'build.log',
    },
    {
      id: 'X2', name: '風格乾淨',
      amplitude: input.lintErrors === 0 ? (input.lintWarnings === 0 ? 100 : 80) : 0,
      status: input.lintErrors === 0 ? (input.lintWarnings === 0 ? 'PASS' : 'WARN') : 'FAIL',
      evidence: 'lint.log',
    },
    {
      id: 'X3', name: '沒踩紅線',
      amplitude: input.redlineViolations === 0 ? 100 : 0,
      status: input.redlineViolations === 0 ? 'PASS' : 'FAIL',
      evidence: 'redline.log',
    },
    {
      id: 'X4', name: '安全無虞',
      amplitude: input.securityCritical === 0 && input.securityHigh === 0 ? 100 : 0,
      status: input.securityCritical === 0 && input.securityHigh === 0 ? 'PASS' : 'FAIL',
      evidence: 'audit.log',
    },
  ];
  return finalizeChannel('CH2_X', '程式碼品質', readings);
}

/** CH3 (Z軸) 真實波形 */
export function buildZChannel(input: {
  fraudCount: number;           // Z1: 詐欺掃描
  iavPass: boolean;             // Z2: IAV 五問
  bldsScore: number;            // Z3: BLDS 0-5
  bldsMinimum: number;
  traceability: boolean;        // Z4: 追溯鏈
}): WaveformChannel {
  const readings: ChannelReading[] = [
    {
      id: 'Z1', name: '有沒有造假',
      amplitude: input.fraudCount === 0 ? 100 : 0,
      status: input.fraudCount === 0 ? 'PASS' : 'FAIL',
      evidence: 'fraud.log',
    },
    {
      id: 'Z2', name: '真實性五問',
      amplitude: input.iavPass ? 100 : 0,
      status: input.iavPass ? 'PASS' : 'FAIL',
      evidence: 'iav.log',
    },
    {
      id: 'Z3', name: '真實度評分',
      amplitude: (input.bldsScore / 5) * 100,
      status: input.bldsScore >= input.bldsMinimum ? 'PASS' : 'FAIL',
      evidence: 'blds.log',
      detail: `BLDS=${input.bldsScore}/5 (min=${input.bldsMinimum})`,
    },
    {
      id: 'Z4', name: '資料來源',
      amplitude: input.traceability ? 100 : 0,
      status: input.traceability ? 'PASS' : 'WARN',
      evidence: 'datasource.log',
    },
  ];
  return finalizeChannel('CH3_Z', '真實度', readings);
}

// =============================================================================
// Helpers
// =============================================================================

function finalizeChannel(
  channel: WaveformChannel['channel'],
  name: string,
  readings: ChannelReading[]
): WaveformChannel {
  const hasAnyFail = readings.some(r => r.status === 'FAIL');
  const hasAnyWarn = readings.some(r => r.status === 'WARN');
  const score = readings.length > 0
    ? readings.reduce((sum, r) => sum + r.amplitude, 0) / readings.length
    : 0;

  return {
    channel,
    name,
    readings,
    overall: hasAnyFail ? 'FAIL' : hasAnyWarn ? 'WARN' : 'PASS',
    score: Math.round(score * 100) / 100,
  };
}

// =============================================================================
// Full Report
// =============================================================================

export function buildWaveformReport(
  y: WaveformChannel,
  x: WaveformChannel,
  z: WaveformChannel,
): WaveformReport {
  const allPass = y.overall === 'PASS' && x.overall === 'PASS' && z.overall === 'PASS';
  // Y:X:Z = 40:30:30
  const compositeScore = Math.round((y.score * 0.4 + x.score * 0.3 + z.score * 0.3) * 100) / 100;

  return {
    timestamp: new Date().toISOString(),
    channels: [y, x, z],
    allPass,
    compositeScore,
  };
}
