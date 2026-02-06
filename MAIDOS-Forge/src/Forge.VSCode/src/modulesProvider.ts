// MAIDOS-Forge VS Code Extension - Modules Tree Provider
// Code-QC v2.2B Compliant

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';

interface ForgeModule {
    name: string;
    language: string;
    version: string;
    path: string;
    dependencies: string[];
}

interface ForgeProject {
    name: string;
    version: string;
    modules: ForgeModule[];
}

export class ForgeModulesProvider implements vscode.TreeDataProvider<ModuleItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<ModuleItem | undefined | null | void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    private project: ForgeProject | null = null;

    constructor() {
        this.loadProject();
    }

    refresh(): void {
        this.loadProject();
        this._onDidChangeTreeData.fire();
    }

    private loadProject(): void {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) {
            this.project = null;
            return;
        }

        const forgeJsonPath = path.join(workspaceFolders[0].uri.fsPath, 'forge.json');
        if (!fs.existsSync(forgeJsonPath)) {
            this.project = null;
            return;
        }

        try {
            const content = fs.readFileSync(forgeJsonPath, 'utf-8');
            const config = JSON.parse(content);

            this.project = {
                name: config.name || 'Unknown',
                version: config.version || '0.0.0',
                modules: []
            };

            // Scan for modules
            this.scanModules(workspaceFolders[0].uri.fsPath);
        } catch (error) {
            console.error('Failed to load forge.json:', error);
            this.project = null;
        }
    }

    private scanModules(rootPath: string): void {
        if (!this.project) return;

        const entries = fs.readdirSync(rootPath, { withFileTypes: true });
        
        for (const entry of entries) {
            if (!entry.isDirectory()) continue;
            if (entry.name.startsWith('.') || entry.name === 'node_modules' || 
                entry.name === 'build' || entry.name === 'output') continue;

            const moduleJsonPath = path.join(rootPath, entry.name, 'module.json');
            if (fs.existsSync(moduleJsonPath)) {
                try {
                    const content = fs.readFileSync(moduleJsonPath, 'utf-8');
                    const moduleConfig = JSON.parse(content);

                    this.project.modules.push({
                        name: moduleConfig.name || entry.name,
                        language: moduleConfig.language || 'unknown',
                        version: moduleConfig.version || '0.0.0',
                        path: path.join(rootPath, entry.name),
                        dependencies: moduleConfig.dependencies || []
                    });
                } catch (error) {
                    console.error(`Failed to load module ${entry.name}:`, error);
                }
            }
        }
    }

    getTreeItem(element: ModuleItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: ModuleItem): Thenable<ModuleItem[]> {
        if (!this.project) {
            return Promise.resolve([]);
        }

        if (!element) {
            // Root level - return modules
            return Promise.resolve(
                this.project.modules.map(m => new ModuleItem(
                    m.name,
                    m.language,
                    m.version,
                    m.path,
                    m.dependencies,
                    vscode.TreeItemCollapsibleState.Collapsed
                ))
            );
        }

        // Module children - return dependencies
        if (element.dependencies.length > 0) {
            return Promise.resolve(
                element.dependencies.map(dep => new ModuleItem(
                    dep,
                    'dependency',
                    '',
                    '',
                    [],
                    vscode.TreeItemCollapsibleState.None
                ))
            );
        }

        return Promise.resolve([]);
    }
}

export class ModuleItem extends vscode.TreeItem {
    constructor(
        public readonly id: string,
        public readonly language: string,
        public readonly version: string,
        public readonly modulePath: string,
        public readonly dependencies: string[],
        public readonly collapsibleState: vscode.TreeItemCollapsibleState
    ) {
        super(id, collapsibleState);

        if (language === 'dependency') {
            this.label = `→ ${id}`;
            this.iconPath = new vscode.ThemeIcon('arrow-right');
            this.contextValue = 'dependency';
        } else {
            this.label = id;
            this.description = `${language} v${version}`;
            this.tooltip = `${id} (${language} v${version})\n${modulePath}`;
            this.iconPath = this.getLanguageIcon(language);
            this.contextValue = 'module';

            if (modulePath) {
                this.resourceUri = vscode.Uri.file(modulePath);
                this.command = {
                    command: 'revealInExplorer',
                    title: 'Open Module',
                    arguments: [vscode.Uri.file(modulePath)]
                };
            }
        }
    }

    private getLanguageIcon(language: string): vscode.ThemeIcon {
        const iconMap: Record<string, string> = {
            'c': 'symbol-file',
            'cpp': 'symbol-file',
            'c++': 'symbol-file',
            'csharp': 'symbol-namespace',
            'c#': 'symbol-namespace',
            'rust': 'symbol-class',
            'go': 'symbol-interface',
            'python': 'symbol-method',
            'typescript': 'symbol-field',
            'javascript': 'symbol-field',
            'java': 'symbol-class',
            'kotlin': 'symbol-class',
            'swift': 'symbol-class',
            'zig': 'symbol-event',
            'nim': 'symbol-event',
            'haskell': 'symbol-function',
            'julia': 'symbol-variable',
            'lua': 'symbol-key',
            'ruby': 'symbol-color'
        };

        const iconName = iconMap[language.toLowerCase()] || 'symbol-misc';
        return new vscode.ThemeIcon(iconName);
    }
}
