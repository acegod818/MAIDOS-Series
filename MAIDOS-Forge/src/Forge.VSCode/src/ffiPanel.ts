// MAIDOS-Forge VS Code Extension - FFI Inspector Panel
// Code-QC v2.2B Compliant

import * as vscode from 'vscode';
import * as cp from 'child_process';

interface FfiFunction {
    name: string;
    returnType: string;
    parameters: { name: string; type: string }[];
    module: string;
    language: string;
}

export class ForgeFfiPanel {
    public static currentPanel: ForgeFfiPanel | undefined;
    public static readonly viewType = 'forgeFfi';

    private readonly _panel: vscode.WebviewPanel;
    private readonly _extensionUri: vscode.Uri;
    private _disposables: vscode.Disposable[] = [];

    public static createOrShow(extensionUri: vscode.Uri): void {
        const column = vscode.window.activeTextEditor
            ? vscode.window.activeTextEditor.viewColumn
            : undefined;

        if (ForgeFfiPanel.currentPanel) {
            ForgeFfiPanel.currentPanel._panel.reveal(column);
            ForgeFfiPanel.currentPanel._update();
            return;
        }

        const panel = vscode.window.createWebviewPanel(
            ForgeFfiPanel.viewType,
            'Forge FFI Inspector',
            column || vscode.ViewColumn.One,
            {
                enableScripts: true,
                retainContextWhenHidden: true
            }
        );

        ForgeFfiPanel.currentPanel = new ForgeFfiPanel(panel, extensionUri);
    }

    private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {
        this._panel = panel;
        this._extensionUri = extensionUri;

        this._update();

        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

        this._panel.webview.onDidReceiveMessage(
            message => {
                switch (message.command) {
                    case 'refresh':
                        this._update();
                        break;
                    case 'generateBinding':
                        this._generateBinding(message.func, message.targetLang);
                        break;
                    case 'copyToClipboard':
                        vscode.env.clipboard.writeText(message.text);
                        vscode.window.showInformationMessage('Copied to clipboard');
                        break;
                }
            },
            null,
            this._disposables
        );
    }

    public dispose(): void {
        ForgeFfiPanel.currentPanel = undefined;

        this._panel.dispose();

        while (this._disposables.length) {
            const x = this._disposables.pop();
            if (x) {
                x.dispose();
            }
        }
    }

    private async _update(): Promise<void> {
        const webview = this._panel.webview;
        const ffiData = await this._getFfiData();
        this._panel.webview.html = this._getHtmlForWebview(webview, ffiData);
    }

    private async _getFfiData(): Promise<FfiFunction[]> {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) {
            return [];
        }

        const forge = vscode.workspace.getConfiguration('forge').get('executablePath') || 'forge';

        return new Promise((resolve) => {
            const process = cp.spawn(forge as string, ['ffi', '--json'], {
                cwd: workspaceFolders[0].uri.fsPath,
                shell: true
            });

            let output = '';

            process.stdout?.on('data', (data) => {
                output += data.toString();
            });

            process.on('close', () => {
                try {
                    const data = JSON.parse(output);
                    resolve(data.functions || []);
                } catch {
                    // Return demo data for development
                    resolve(this._getDemoData());
                }
            });

            process.on('error', () => {
                resolve(this._getDemoData());
            });

            // Timeout
            setTimeout(() => {
                process.kill();
                resolve(this._getDemoData());
            }, 5000);
        });
    }

    private _getDemoData(): FfiFunction[] {
        return [
            {
                name: 'calculate_hash',
                returnType: 'uint64_t',
                parameters: [
                    { name: 'data', type: 'const uint8_t*' },
                    { name: 'len', type: 'size_t' }
                ],
                module: 'core',
                language: 'rust'
            },
            {
                name: 'create_context',
                returnType: 'Context*',
                parameters: [],
                module: 'core',
                language: 'rust'
            },
            {
                name: 'process_buffer',
                returnType: 'int32_t',
                parameters: [
                    { name: 'ctx', type: 'Context*' },
                    { name: 'buffer', type: 'uint8_t*' },
                    { name: 'size', type: 'size_t' }
                ],
                module: 'core',
                language: 'rust'
            },
            {
                name: 'destroy_context',
                returnType: 'void',
                parameters: [
                    { name: 'ctx', type: 'Context*' }
                ],
                module: 'core',
                language: 'rust'
            }
        ];
    }

    private _generateBinding(func: FfiFunction, targetLang: string): void {
        let binding = '';

        switch (targetLang) {
            case 'csharp':
                binding = this._generateCSharpBinding(func);
                break;
            case 'python':
                binding = this._generatePythonBinding(func);
                break;
            case 'go':
                binding = this._generateGoBinding(func);
                break;
            default:
                binding = `// Binding for ${func.name}`;
        }

        // Create new document with binding
        vscode.workspace.openTextDocument({
            content: binding,
            language: targetLang === 'csharp' ? 'csharp' : targetLang
        }).then(doc => {
            vscode.window.showTextDocument(doc);
        });
    }

    private _generateCSharpBinding(func: FfiFunction): string {
        const typeMap: Record<string, string> = {
            'int32_t': 'int',
            'uint32_t': 'uint',
            'int64_t': 'long',
            'uint64_t': 'ulong',
            'size_t': 'nuint',
            'void': 'void',
            'const uint8_t*': 'IntPtr',
            'uint8_t*': 'IntPtr',
            'Context*': 'IntPtr'
        };

        const returnType = typeMap[func.returnType] || 'IntPtr';
        const params = func.parameters.map(p => 
            `${typeMap[p.type] || 'IntPtr'} ${p.name}`
        ).join(', ');

        return `// Auto-generated C# binding for ${func.name}
using System.Runtime.InteropServices;

public static partial class NativeBindings
{
    [DllImport("${func.module}", CallingConvention = CallingConvention.Cdecl)]
    public static extern ${returnType} ${func.name}(${params});
}
`;
    }

    private _generatePythonBinding(func: FfiFunction): string {
        const typeMap: Record<string, string> = {
            'int32_t': 'ctypes.c_int32',
            'uint32_t': 'ctypes.c_uint32',
            'int64_t': 'ctypes.c_int64',
            'uint64_t': 'ctypes.c_uint64',
            'size_t': 'ctypes.c_size_t',
            'void': 'None',
            'const uint8_t*': 'ctypes.POINTER(ctypes.c_uint8)',
            'uint8_t*': 'ctypes.POINTER(ctypes.c_uint8)',
            'Context*': 'ctypes.c_void_p'
        };

        const returnType = typeMap[func.returnType] || 'ctypes.c_void_p';
        const argTypes = func.parameters.map(p => 
            typeMap[p.type] || 'ctypes.c_void_p'
        ).join(', ');

        return `# Auto-generated Python binding for ${func.name}
import ctypes

lib = ctypes.CDLL("./${func.module}.so")

lib.${func.name}.restype = ${returnType}
lib.${func.name}.argtypes = [${argTypes}]

def ${func.name}(${func.parameters.map(p => p.name).join(', ')}):
    return lib.${func.name}(${func.parameters.map(p => p.name).join(', ')})
`;
    }

    private _generateGoBinding(func: FfiFunction): string {
        const typeMap: Record<string, string> = {
            'int32_t': 'C.int32_t',
            'uint32_t': 'C.uint32_t',
            'int64_t': 'C.int64_t',
            'uint64_t': 'C.uint64_t',
            'size_t': 'C.size_t',
            'void': '',
            'const uint8_t*': '*C.uint8_t',
            'uint8_t*': '*C.uint8_t',
            'Context*': '*C.Context'
        };

        const goTypeMap: Record<string, string> = {
            'int32_t': 'int32',
            'uint32_t': 'uint32',
            'int64_t': 'int64',
            'uint64_t': 'uint64',
            'size_t': 'uint',
            'void': '',
            'const uint8_t*': '*byte',
            'uint8_t*': '*byte',
            'Context*': 'unsafe.Pointer'
        };

        const returnType = goTypeMap[func.returnType] || 'unsafe.Pointer';
        const params = func.parameters.map(p => 
            `${p.name} ${goTypeMap[p.type] || 'unsafe.Pointer'}`
        ).join(', ');
        const cParams = func.parameters.map(p => 
            `(${typeMap[p.type] || 'C.void'})(${p.name})`
        ).join(', ');

        const funcName = func.name.split('_').map(s => 
            s.charAt(0).toUpperCase() + s.slice(1)
        ).join('');

        return `// Auto-generated Go binding for ${func.name}
package main

/*
#cgo LDFLAGS: -L. -l${func.module}
#include "${func.module}.h"
*/
import "C"
import "unsafe"

func ${funcName}(${params}) ${returnType} {
    return ${returnType}(C.${func.name}(${cParams}))
}
`;
    }

    private _getHtmlForWebview(webview: vscode.Webview, functions: FfiFunction[]): string {
        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Forge FFI Inspector</title>
    <style>
        body {
            margin: 0;
            padding: 20px;
            background: var(--vscode-editor-background);
            color: var(--vscode-editor-foreground);
            font-family: var(--vscode-font-family);
        }
        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 20px;
        }
        h2 { margin: 0; color: var(--vscode-textLink-foreground); }
        button {
            background: var(--vscode-button-background);
            color: var(--vscode-button-foreground);
            border: none;
            padding: 6px 14px;
            cursor: pointer;
            border-radius: 2px;
            margin-left: 4px;
        }
        button:hover { background: var(--vscode-button-hoverBackground); }
        .function-card {
            background: var(--vscode-editor-inactiveSelectionBackground);
            border: 1px solid var(--vscode-panel-border);
            border-radius: 4px;
            padding: 12px;
            margin-bottom: 12px;
        }
        .function-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .function-name {
            font-family: var(--vscode-editor-font-family);
            font-size: 14px;
            color: var(--vscode-symbolIcon-functionForeground);
        }
        .function-meta {
            font-size: 11px;
            color: var(--vscode-descriptionForeground);
        }
        .function-signature {
            font-family: var(--vscode-editor-font-family);
            font-size: 12px;
            background: var(--vscode-textCodeBlock-background);
            padding: 8px;
            border-radius: 2px;
            margin-top: 8px;
            overflow-x: auto;
        }
        .return-type { color: var(--vscode-symbolIcon-typeParameterForeground); }
        .param-name { color: var(--vscode-symbolIcon-variableForeground); }
        .param-type { color: var(--vscode-symbolIcon-typeParameterForeground); }
        .actions { margin-top: 8px; }
        .badge {
            display: inline-block;
            padding: 2px 6px;
            border-radius: 2px;
            font-size: 10px;
            background: var(--vscode-badge-background);
            color: var(--vscode-badge-foreground);
        }
        .empty-state {
            text-align: center;
            padding: 40px;
            color: var(--vscode-descriptionForeground);
        }
    </style>
</head>
<body>
    <div class="header">
        <h2>FFI Interfaces</h2>
        <button onclick="refresh()">↻ Refresh</button>
    </div>

    <div id="content">
        ${functions.length === 0 ? `
            <div class="empty-state">
                <p>No FFI functions found</p>
                <p>Export functions with extern "C" to see them here</p>
            </div>
        ` : functions.map(f => `
            <div class="function-card">
                <div class="function-header">
                    <span class="function-name">${f.name}</span>
                    <span class="function-meta">
                        <span class="badge">${f.language}</span>
                        <span class="badge">${f.module}</span>
                    </span>
                </div>
                <div class="function-signature">
                    <span class="return-type">${f.returnType}</span> 
                    <strong>${f.name}</strong>(${f.parameters.map(p => 
                        `<span class="param-type">${p.type}</span> <span class="param-name">${p.name}</span>`
                    ).join(', ')})
                </div>
                <div class="actions">
                    <button onclick="generateBinding('${encodeURIComponent(JSON.stringify(f))}', 'csharp')">C# Binding</button>
                    <button onclick="generateBinding('${encodeURIComponent(JSON.stringify(f))}', 'python')">Python Binding</button>
                    <button onclick="generateBinding('${encodeURIComponent(JSON.stringify(f))}', 'go')">Go Binding</button>
                    <button onclick="copySignature('${f.returnType} ${f.name}(${f.parameters.map(p => p.type + ' ' + p.name).join(', ')})')">Copy</button>
                </div>
            </div>
        `).join('')}
    </div>

    <script>
        const vscode = acquireVsCodeApi();

        function refresh() {
            vscode.postMessage({ command: 'refresh' });
        }

        function generateBinding(funcJson, targetLang) {
            const func = JSON.parse(decodeURIComponent(funcJson));
            vscode.postMessage({ command: 'generateBinding', func, targetLang });
        }

        function copySignature(text) {
            vscode.postMessage({ command: 'copyToClipboard', text });
        }
    </script>
</body>
</html>`;
    }
}
