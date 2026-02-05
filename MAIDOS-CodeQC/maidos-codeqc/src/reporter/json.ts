/**
 * JSON Reporter
 * 輸出結構化 JSON 報告
 */

import type { AnalysisResult, Reporter } from '../types.js';

export const jsonReporter: Reporter = {
  name: 'json',
  
  report(result: AnalysisResult): string {
    return JSON.stringify(result, null, 2);
  },
};

export default jsonReporter;
