// MAIDOS-Forge VS Code Extension - Diagnostics
// Code-QC v2.2B Compliant

import * as vscode from 'vscode';
import * as path from 'path';

interface ParsedError {
    file: string;
    line: number;
    column: number;
    severity: vscode.DiagnosticSeverity;
    message: string;
    source: string;
}

export class ForgeDiagnostics implements vscode.Disposable {
    private diagnosticCollection: vscode.DiagnosticCollection;
    private decorationType: vscode.TextEditorDecorationType;

    constructor() {
        this.diagnosticCollection = vscode.languages.createDiagnosticCollection('forge');
        
        // Create inline error decoration
        this.decorationType = vscode.window.createTextEditorDecorationType({
            after: {
                margin: '0 0 0 1em',
                color: new vscode.ThemeColor('editorError.foreground')
            },
            isWholeLine: true
        });
    }

    dispose(): void {
        this.diagnosticCollection.dispose();
        this.decorationType.dispose();
    }

    clear(): void {
        this.diagnosticCollection.clear();
        
        // Clear decorations from all visible editors
        for (const editor of vscode.window.visibleTextEditors) {
            editor.setDecorations(this.decorationType, []);
        }
    }

    parseOutput(output: string): void {
        this.clear();

        const errors = this.parseErrors(output);
        const diagnosticsMap = new Map<string, vscode.Diagnostic[]>();

        for (const error of errors) {
            const uri = this.resolveUri(error.file);
            if (!uri) continue;

            const key = uri.toString();
            if (!diagnosticsMap.has(key)) {
                diagnosticsMap.set(key, []);
            }

            const range = new vscode.Range(
                Math.max(0, error.line - 1),
                Math.max(0, error.column - 1),
                Math.max(0, error.line - 1),
                1000 // End of line
            );

            const diagnostic = new vscode.Diagnostic(
                range,
                error.message,
                error.severity
            );
            diagnostic.source = error.source;

            diagnosticsMap.get(key)!.push(diagnostic);
        }

        // Update diagnostics
        for (const [uriString, diagnostics] of diagnosticsMap) {
            this.diagnosticCollection.set(vscode.Uri.parse(uriString), diagnostics);
        }

        // Update inline decorations
        this.updateInlineDecorations(errors);
    }

    private parseErrors(output: string): ParsedError[] {
        const errors: ParsedError[] = [];
        const lines = output.split('\n');

        // Common error patterns for various languages
        const patterns = [
            // GCC/Clang: file:line:col: error: message
            /^(.+):(\d+):(\d+):\s*(error|warning|note):\s*(.+)$/,
            // Rust: error[E0001]: message --> file:line:col
            /^(error|warning)\[.*?\]:\s*(.+)\s*-->\s*(.+):(\d+):(\d+)/,
            // .NET: file(line,col): error CS0001: message
            /^(.+)\((\d+),(\d+)\):\s*(error|warning)\s+\w+:\s*(.+)$/,
            // Go: file:line:col: message
            /^(.+):(\d+):(\d+):\s*(.+)$/,
            // Python: File "file", line N
            /^File "(.+)", line (\d+)/,
            // TypeScript: file(line,col): error TS0001: message
            /^(.+)\((\d+),(\d+)\):\s*error\s+TS\d+:\s*(.+)$/,
            // General: file:line: message
            /^(.+):(\d+):\s*(error|warning|Error|Warning):\s*(.+)$/
        ];

        for (const line of lines) {
            for (const pattern of patterns) {
                const match = line.match(pattern);
                if (match) {
                    const error = this.extractError(match, pattern);
                    if (error) {
                        errors.push(error);
                    }
                    break;
                }
            }
        }

        return errors;
    }

    private extractError(match: RegExpMatchArray, pattern: RegExp): ParsedError | null {
        const patternStr = pattern.toString();

        // GCC/Clang style
        if (patternStr.includes('error|warning|note')) {
            const severity = match[4] === 'error' ? vscode.DiagnosticSeverity.Error :
                            match[4] === 'warning' ? vscode.DiagnosticSeverity.Warning :
                            vscode.DiagnosticSeverity.Information;

            return {
                file: match[1],
                line: parseInt(match[2]),
                column: parseInt(match[3]),
                severity,
                message: match[5],
                source: 'forge'
            };
        }

        // Rust style (reversed order)
        if (patternStr.includes('-->')) {
            const severity = match[1] === 'error' ? vscode.DiagnosticSeverity.Error :
                            vscode.DiagnosticSeverity.Warning;

            return {
                file: match[3],
                line: parseInt(match[4]),
                column: parseInt(match[5]),
                severity,
                message: match[2],
                source: 'rustc'
            };
        }

        // .NET/TypeScript style with parentheses
        if (patternStr.includes('\\(\\d+,\\d+\\)')) {
            const severityStr = match[4]?.toLowerCase() || 'error';
            const severity = severityStr === 'error' ? vscode.DiagnosticSeverity.Error :
                            vscode.DiagnosticSeverity.Warning;

            return {
                file: match[1],
                line: parseInt(match[2]),
                column: parseInt(match[3]),
                severity,
                message: match[5] || match[4],
                source: 'dotnet'
            };
        }

        // Generic fallback
        if (match.length >= 3) {
            return {
                file: match[1],
                line: parseInt(match[2]),
                column: match[3] ? parseInt(match[3]) : 1,
                severity: vscode.DiagnosticSeverity.Error,
                message: match[match.length - 1] || 'Unknown error',
                source: 'forge'
            };
        }

        return null;
    }

    private resolveUri(file: string): vscode.Uri | null {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) return null;

        // If absolute path
        if (path.isAbsolute(file)) {
            return vscode.Uri.file(file);
        }

        // Try relative to workspace
        for (const folder of workspaceFolders) {
            const fullPath = path.join(folder.uri.fsPath, file);
            // We can't check if file exists synchronously, so just return the URI
            return vscode.Uri.file(fullPath);
        }

        return null;
    }

    private async updateInlineDecorations(errors: ParsedError[]): Promise<void> {
        const config = vscode.workspace.getConfiguration('forge');
        if (!config.get('showInlineErrors')) return;

        // Group errors by file
        const errorsByFile = new Map<string, ParsedError[]>();
        for (const error of errors) {
            const uri = this.resolveUri(error.file);
            if (!uri) continue;

            const key = uri.toString();
            if (!errorsByFile.has(key)) {
                errorsByFile.set(key, []);
            }
            errorsByFile.get(key)!.push(error);
        }

        // Apply decorations to visible editors
        for (const editor of vscode.window.visibleTextEditors) {
            const uriString = editor.document.uri.toString();
            const fileErrors = errorsByFile.get(uriString);

            if (!fileErrors || fileErrors.length === 0) {
                editor.setDecorations(this.decorationType, []);
                continue;
            }

            const decorations: vscode.DecorationOptions[] = fileErrors
                .filter(e => e.severity === vscode.DiagnosticSeverity.Error)
                .map(e => ({
                    range: new vscode.Range(e.line - 1, 0, e.line - 1, 1000),
                    renderOptions: {
                        after: {
                            contentText: ` ← ${e.message}`,
                            color: new vscode.ThemeColor('editorError.foreground')
                        }
                    }
                }));

            editor.setDecorations(this.decorationType, decorations);
        }
    }
}
