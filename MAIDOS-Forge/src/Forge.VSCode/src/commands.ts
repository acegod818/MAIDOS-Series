// MAIDOS-Forge VS Code Extension - Commands
// Code-QC v2.2B Compliant

import * as vscode from 'vscode';
import * as path from 'path';
import * as cp from 'child_process';
import { ForgeDiagnostics } from './diagnostics';

export class ForgeCommands {
    private outputChannel: vscode.OutputChannel;
    private diagnostics: ForgeDiagnostics;
    private watchProcess: cp.ChildProcess | null = null;

    constructor(outputChannel: vscode.OutputChannel, diagnostics: ForgeDiagnostics) {
        this.outputChannel = outputChannel;
        this.diagnostics = diagnostics;
    }

    private getForgeExecutable(): string {
        const config = vscode.workspace.getConfiguration('forge');
        return config.get('executablePath') || 'forge';
    }

    private getWorkspaceFolder(): string | undefined {
        const folders = vscode.workspace.workspaceFolders;
        return folders?.[0]?.uri.fsPath;
    }

    private async runForgeCommand(args: string[], showOutput: boolean = true): Promise<{ success: boolean; output: string }> {
        const cwd = this.getWorkspaceFolder();
        if (!cwd) {
            vscode.window.showErrorMessage('No workspace folder open');
            return { success: false, output: '' };
        }

        const forge = this.getForgeExecutable();
        const fullCommand = `${forge} ${args.join(' ')}`;

        if (showOutput) {
            this.outputChannel.show();
            this.outputChannel.appendLine(`> ${fullCommand}`);
        }

        return new Promise((resolve) => {
            const process = cp.spawn(forge, args, { cwd, shell: true });
            let output = '';
            let errorOutput = '';

            process.stdout?.on('data', (data) => {
                const text = data.toString();
                output += text;
                if (showOutput) {
                    this.outputChannel.append(text);
                }
            });

            process.stderr?.on('data', (data) => {
                const text = data.toString();
                errorOutput += text;
                if (showOutput) {
                    this.outputChannel.append(text);
                }
            });

            process.on('close', (code) => {
                const success = code === 0;
                if (showOutput) {
                    this.outputChannel.appendLine(success ? '✓ Done' : `✗ Failed (exit code ${code})`);
                }

                // Parse errors for diagnostics
                if (!success) {
                    this.diagnostics.parseOutput(errorOutput + output);
                } else {
                    this.diagnostics.clear();
                }

                resolve({ success, output: output + errorOutput });
            });

            process.on('error', (err) => {
                if (showOutput) {
                    this.outputChannel.appendLine(`Error: ${err.message}`);
                }
                resolve({ success: false, output: err.message });
            });
        });
    }

    async init(): Promise<void> {
        const cwd = this.getWorkspaceFolder();
        if (!cwd) {
            vscode.window.showErrorMessage('No workspace folder open');
            return;
        }

        // Check if forge.json already exists
        const forgeJson = path.join(cwd, 'forge.json');
        if (require('fs').existsSync(forgeJson)) {
            const overwrite = await vscode.window.showWarningMessage(
                'forge.json already exists. Reinitialize?',
                'Yes', 'No'
            );
            if (overwrite !== 'Yes') return;
        }

        // Get project name
        const name = await vscode.window.showInputBox({
            prompt: 'Project name',
            value: path.basename(cwd),
            validateInput: (value) => {
                if (!value.match(/^[a-z][a-z0-9-]*$/)) {
                    return 'Use lowercase letters, numbers, and hyphens only';
                }
                return null;
            }
        });

        if (!name) return;

        const result = await this.runForgeCommand(['init', name]);
        if (result.success) {
            vscode.window.showInformationMessage(`Forge project "${name}" initialized`);
            vscode.commands.executeCommand('setContext', 'forge.projectLoaded', true);
        }
    }

    async build(): Promise<void> {
        const config = vscode.workspace.getConfiguration('forge');
        const buildConfig = config.get('defaultConfig') || 'Debug';
        const jobs = config.get('parallelJobs') || 0;

        const args = ['build', '-c', buildConfig as string];
        if (jobs > 0) {
            args.push('-j', jobs.toString());
        }

        await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: 'Building project...',
            cancellable: true
        }, async (progress, token) => {
            const result = await this.runForgeCommand(args);
            if (result.success) {
                vscode.window.showInformationMessage('Build successful');
            } else {
                vscode.window.showErrorMessage('Build failed. Check output for details.');
            }
        });
    }

    async rebuild(): Promise<void> {
        await this.clean();
        await this.build();
    }

    async clean(): Promise<void> {
        const result = await this.runForgeCommand(['clean']);
        if (result.success) {
            vscode.window.showInformationMessage('Clean successful');
        }
    }

    async run(): Promise<void> {
        // First build
        const buildResult = await this.runForgeCommand(['build']);
        if (!buildResult.success) {
            vscode.window.showErrorMessage('Build failed. Cannot run.');
            return;
        }

        // Then run
        await this.runForgeCommand(['run']);
    }

    watch(): void {
        if (this.watchProcess) {
            this.watchProcess.kill();
            this.watchProcess = null;
            vscode.window.showInformationMessage('Watch mode stopped');
            return;
        }

        const cwd = this.getWorkspaceFolder();
        if (!cwd) return;

        const forge = this.getForgeExecutable();
        this.outputChannel.show();
        this.outputChannel.appendLine('> forge watch');
        this.outputChannel.appendLine('Watching for file changes... (run command again to stop)');

        this.watchProcess = cp.spawn(forge, ['watch'], { cwd, shell: true });

        this.watchProcess.stdout?.on('data', (data) => {
            this.outputChannel.append(data.toString());
        });

        this.watchProcess.stderr?.on('data', (data) => {
            this.outputChannel.append(data.toString());
        });

        this.watchProcess.on('close', () => {
            this.outputChannel.appendLine('Watch mode ended');
            this.watchProcess = null;
        });

        vscode.window.showInformationMessage('Watch mode started');
    }

    async addModule(): Promise<void> {
        // Get module name
        const name = await vscode.window.showInputBox({
            prompt: 'Module name',
            validateInput: (value) => {
                if (!value.match(/^[a-z][a-z0-9-]*$/)) {
                    return 'Use lowercase letters, numbers, and hyphens only';
                }
                return null;
            }
        });

        if (!name) return;

        // Select language
        const languages = [
            { label: 'C', description: 'Native C' },
            { label: 'C++', description: 'Native C++' },
            { label: 'C#', description: '.NET/Mono' },
            { label: 'Rust', description: 'Rust with Cargo' },
            { label: 'Go', description: 'Go modules' },
            { label: 'Python', description: 'Python/Cython' },
            { label: 'TypeScript', description: 'TypeScript/JavaScript' },
            { label: 'Java', description: 'Java/JVM' },
            { label: 'Kotlin', description: 'Kotlin/JVM' },
            { label: 'Swift', description: 'Swift' },
            { label: 'Zig', description: 'Zig' },
            { label: 'Nim', description: 'Nim' },
            { label: 'Haskell', description: 'Haskell/GHC' },
            { label: 'Julia', description: 'Julia' },
            { label: 'Lua', description: 'Lua/LuaJIT' },
            { label: 'Ruby', description: 'Ruby/mruby' }
        ];

        const selected = await vscode.window.showQuickPick(languages, {
            placeHolder: 'Select language for module'
        });

        if (!selected) return;

        // Create module directory
        const cwd = this.getWorkspaceFolder();
        if (!cwd) return;

        const moduleDir = path.join(cwd, name);
        const fs = require('fs');

        if (fs.existsSync(moduleDir)) {
            vscode.window.showErrorMessage(`Directory "${name}" already exists`);
            return;
        }

        fs.mkdirSync(moduleDir, { recursive: true });

        // Create module.json
        const moduleJson = {
            name: name,
            language: selected.label.toLowerCase(),
            version: "0.1.0",
            sources: ["."],
            dependencies: []
        };

        fs.writeFileSync(
            path.join(moduleDir, 'module.json'),
            JSON.stringify(moduleJson, null, 2)
        );

        // Create template source file
        this.createTemplateSource(moduleDir, name, selected.label);

        vscode.window.showInformationMessage(`Module "${name}" (${selected.label}) created`);
        vscode.commands.executeCommand('forge.refreshModules');
    }

    private createTemplateSource(dir: string, name: string, language: string): void {
        const fs = require('fs');
        const templates: Record<string, { file: string; content: string }> = {
            'C': {
                file: `${name}.c`,
                content: `// ${name} module\n#include <stdio.h>\n\nvoid ${name}_init(void) {\n    printf("${name} initialized\\n");\n}\n`
            },
            'C++': {
                file: `${name}.cpp`,
                content: `// ${name} module\n#include <iostream>\n\nextern "C" void ${name}_init() {\n    std::cout << "${name} initialized" << std::endl;\n}\n`
            },
            'C#': {
                file: `${this.toPascalCase(name)}.cs`,
                content: `// ${name} module\nnamespace ${this.toPascalCase(name)};\n\npublic static class Module {\n    public static void Init() {\n        Console.WriteLine("${name} initialized");\n    }\n}\n`
            },
            'Rust': {
                file: 'lib.rs',
                content: `// ${name} module\n\n#[no_mangle]\npub extern "C" fn ${name.replace(/-/g, '_')}_init() {\n    println!("${name} initialized");\n}\n`
            },
            'Go': {
                file: `${name}.go`,
                content: `// ${name} module\npackage main\n\nimport "C"\nimport "fmt"\n\n//export ${name}_init\nfunc ${name}_init() {\n\tfmt.Println("${name} initialized")\n}\n\nfunc main() {}\n`
            },
            'Python': {
                file: `${name}.py`,
                content: `# ${name} module\n\ndef init():\n    print("${name} initialized")\n\nif __name__ == "__main__":\n    init()\n`
            },
            'TypeScript': {
                file: 'index.ts',
                content: `// ${name} module\n\nexport function init(): void {\n    console.log("${name} initialized");\n}\n`
            }
        };

        const template = templates[language];
        if (template) {
            fs.writeFileSync(path.join(dir, template.file), template.content);
        }
    }

    private toPascalCase(str: string): string {
        return str.split('-').map(s => s.charAt(0).toUpperCase() + s.slice(1)).join('');
    }

    async checkToolchain(): Promise<void> {
        await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: 'Checking toolchains...',
            cancellable: false
        }, async () => {
            const result = await this.runForgeCommand(['toolchain', 'check']);
            vscode.commands.executeCommand('forge.refreshToolchains');
        });
    }

    async buildModule(moduleId: string): Promise<void> {
        const config = vscode.workspace.getConfiguration('forge');
        const buildConfig = config.get('defaultConfig') || 'Debug';

        await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: `Building ${moduleId}...`,
            cancellable: true
        }, async () => {
            await this.runForgeCommand(['build', '-m', moduleId, '-c', buildConfig as string]);
        });
    }

    async openStudio(): Promise<void> {
        const config = vscode.workspace.getConfiguration('forge');
        let studioPath = config.get('studioPath') as string;

        if (!studioPath) {
            // Try to find it in common locations
            const fs = require('fs');
            const possiblePaths = [
                '/usr/local/bin/forge-studio',
                '/usr/bin/forge-studio',
                path.join(require('os').homedir(), '.forge', 'studio', 'Forge.Studio'),
                'C:\\Program Files\\Forge\\Forge.Studio.exe',
                'C:\\Program Files (x86)\\Forge\\Forge.Studio.exe'
            ];

            for (const p of possiblePaths) {
                if (fs.existsSync(p)) {
                    studioPath = p;
                    break;
                }
            }
        }

        if (!studioPath) {
            const action = await vscode.window.showErrorMessage(
                'Forge Studio not found. Would you like to configure the path?',
                'Configure', 'Cancel'
            );
            if (action === 'Configure') {
                vscode.commands.executeCommand('workbench.action.openSettings', 'forge.studioPath');
            }
            return;
        }

        const cwd = this.getWorkspaceFolder();
        cp.spawn(studioPath, cwd ? [cwd] : [], { detached: true, stdio: 'ignore' });
    }
}
