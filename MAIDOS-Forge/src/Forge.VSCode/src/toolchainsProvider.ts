// MAIDOS-Forge VS Code Extension - Toolchains Tree Provider
// Code-QC v2.2B Compliant

import * as vscode from 'vscode';
import * as cp from 'child_process';

interface Toolchain {
    name: string;
    command: string;
    version: string | null;
    available: boolean;
    languages: string[];
}

export class ForgeToolchainsProvider implements vscode.TreeDataProvider<ToolchainItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<ToolchainItem | undefined | null | void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    private toolchains: Toolchain[] = [];
    private isChecking = false;

    constructor() {
        this.initToolchains();
    }

    private initToolchains(): void {
        this.toolchains = [
            { name: '.NET SDK', command: 'dotnet', version: null, available: false, languages: ['C#', 'F#'] },
            { name: 'Rust', command: 'rustc', version: null, available: false, languages: ['Rust'] },
            { name: 'Cargo', command: 'cargo', version: null, available: false, languages: ['Rust'] },
            { name: 'GCC', command: 'gcc', version: null, available: false, languages: ['C', 'C++'] },
            { name: 'Clang', command: 'clang', version: null, available: false, languages: ['C', 'C++', 'Obj-C'] },
            { name: 'Go', command: 'go', version: null, available: false, languages: ['Go'] },
            { name: 'Python', command: 'python3', version: null, available: false, languages: ['Python'] },
            { name: 'Node.js', command: 'node', version: null, available: false, languages: ['JavaScript', 'TypeScript'] },
            { name: 'Java', command: 'javac', version: null, available: false, languages: ['Java'] },
            { name: 'Kotlin', command: 'kotlinc', version: null, available: false, languages: ['Kotlin'] },
            { name: 'Zig', command: 'zig', version: null, available: false, languages: ['Zig'] },
            { name: 'Nim', command: 'nim', version: null, available: false, languages: ['Nim'] },
            { name: 'GHC', command: 'ghc', version: null, available: false, languages: ['Haskell'] },
            { name: 'Julia', command: 'julia', version: null, available: false, languages: ['Julia'] },
            { name: 'Ruby', command: 'ruby', version: null, available: false, languages: ['Ruby'] },
            { name: 'Lua', command: 'lua', version: null, available: false, languages: ['Lua'] },
            { name: 'Swift', command: 'swiftc', version: null, available: false, languages: ['Swift'] },
            { name: 'Erlang', command: 'erl', version: null, available: false, languages: ['Erlang'] },
            { name: 'Elixir', command: 'elixir', version: null, available: false, languages: ['Elixir'] },
            { name: 'NASM', command: 'nasm', version: null, available: false, languages: ['Assembly'] }
        ];

        this.checkAllToolchains();
    }

    refresh(): void {
        this.checkAllToolchains();
    }

    private async checkAllToolchains(): Promise<void> {
        if (this.isChecking) return;
        this.isChecking = true;

        const promises = this.toolchains.map(tc => this.checkToolchain(tc));
        await Promise.all(promises);

        this.isChecking = false;
        this._onDidChangeTreeData.fire();
    }

    private async checkToolchain(toolchain: Toolchain): Promise<void> {
        return new Promise((resolve) => {
            const versionArg = toolchain.command === 'dotnet' ? '--version' :
                               toolchain.command === 'go' ? 'version' :
                               toolchain.command === 'julia' ? '--version' :
                               toolchain.command === 'erl' ? '+V' :
                               '--version';

            const process = cp.spawn(toolchain.command, [versionArg], { shell: true });
            let output = '';

            process.stdout?.on('data', (data) => {
                output += data.toString();
            });

            process.stderr?.on('data', (data) => {
                output += data.toString();
            });

            process.on('close', (code) => {
                if (code === 0 || output.trim()) {
                    toolchain.available = true;
                    toolchain.version = this.parseVersion(output, toolchain.command);
                } else {
                    toolchain.available = false;
                    toolchain.version = null;
                }
                resolve();
            });

            process.on('error', () => {
                toolchain.available = false;
                toolchain.version = null;
                resolve();
            });

            // Timeout after 5 seconds
            setTimeout(() => {
                process.kill();
                toolchain.available = false;
                toolchain.version = null;
                resolve();
            }, 5000);
        });
    }

    private parseVersion(output: string, command: string): string {
        const lines = output.split('\n').filter(l => l.trim());
        if (lines.length === 0) return 'unknown';

        const firstLine = lines[0].trim();

        // Try to extract version number
        const versionMatch = firstLine.match(/(\d+\.\d+(?:\.\d+)?)/);
        if (versionMatch) {
            return `v${versionMatch[1]}`;
        }

        return firstLine.length > 30 ? firstLine.substring(0, 30) + '...' : firstLine;
    }

    getTreeItem(element: ToolchainItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: ToolchainItem): Thenable<ToolchainItem[]> {
        if (element) {
            return Promise.resolve([]);
        }

        // Sort: available first, then by name
        const sorted = [...this.toolchains].sort((a, b) => {
            if (a.available !== b.available) {
                return a.available ? -1 : 1;
            }
            return a.name.localeCompare(b.name);
        });

        return Promise.resolve(
            sorted.map(tc => new ToolchainItem(tc))
        );
    }
}

export class ToolchainItem extends vscode.TreeItem {
    constructor(toolchain: Toolchain) {
        super(toolchain.name, vscode.TreeItemCollapsibleState.None);

        if (toolchain.available) {
            this.description = toolchain.version || 'available';
            this.iconPath = new vscode.ThemeIcon('check', new vscode.ThemeColor('testing.iconPassed'));
            this.tooltip = `${toolchain.name} (${toolchain.command})\n${toolchain.version}\nLanguages: ${toolchain.languages.join(', ')}`;
        } else {
            this.description = 'not found';
            this.iconPath = new vscode.ThemeIcon('close', new vscode.ThemeColor('testing.iconFailed'));
            this.tooltip = `${toolchain.name} (${toolchain.command})\nNot installed\nLanguages: ${toolchain.languages.join(', ')}`;
        }

        this.contextValue = toolchain.available ? 'toolchain-available' : 'toolchain-missing';
    }
}
