#!/usr/bin/env node
/**
 * MAIDOS CodeQC CLI
 *
 * 支援功能：
 * - 單檔案 / 批量資料夾掃描
 * - 軟配置選擇分析類型 (可複選)
 * - 多種輸出格式 (Console/JSON/HTML)
 */
type AnalysisCategory = 'security' | 'structure' | 'quality';
interface AnalysisConfig {
    categories: Set<AnalysisCategory>;
    rules: {
        security: {
            credentials: boolean;
            injection: boolean;
            auditLogs: boolean;
            errorHandling: boolean;
            securityDisable: boolean;
            vulnerabilities: boolean;
            resources: boolean;
            plaintext: boolean;
        };
        structure: {
            longFunction: boolean;
            deepNesting: boolean;
            globalState: boolean;
            longParams: boolean;
            copyPaste: boolean;
        };
        quality: {
            magicNumbers: boolean;
            naming: boolean;
            todos: boolean;
            comments: boolean;
            dependencies: boolean;
        };
    };
}
declare const DEFAULT_CONFIG: AnalysisConfig;
declare function loadFiles(targetPath: string): Array<{
    path: string;
    content: string;
}>;

export { type AnalysisCategory, type AnalysisConfig, DEFAULT_CONFIG, loadFiles };
