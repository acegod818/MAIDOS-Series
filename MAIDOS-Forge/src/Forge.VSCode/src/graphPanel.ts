// MAIDOS-Forge VS Code Extension - Dependency Graph Panel
// Code-QC v2.2B Compliant

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import * as cp from 'child_process';

export class ForgeGraphPanel {
    public static currentPanel: ForgeGraphPanel | undefined;
    public static readonly viewType = 'forgeGraph';

    private readonly _panel: vscode.WebviewPanel;
    private readonly _extensionUri: vscode.Uri;
    private _disposables: vscode.Disposable[] = [];

    public static createOrShow(extensionUri: vscode.Uri): void {
        const column = vscode.window.activeTextEditor
            ? vscode.window.activeTextEditor.viewColumn
            : undefined;

        if (ForgeGraphPanel.currentPanel) {
            ForgeGraphPanel.currentPanel._panel.reveal(column);
            ForgeGraphPanel.currentPanel._update();
            return;
        }

        const panel = vscode.window.createWebviewPanel(
            ForgeGraphPanel.viewType,
            'Forge Dependency Graph',
            column || vscode.ViewColumn.One,
            {
                enableScripts: true,
                retainContextWhenHidden: true
            }
        );

        ForgeGraphPanel.currentPanel = new ForgeGraphPanel(panel, extensionUri);
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
                    case 'openModule':
                        this._openModule(message.module);
                        break;
                }
            },
            null,
            this._disposables
        );
    }

    public dispose(): void {
        ForgeGraphPanel.currentPanel = undefined;

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
        const graphData = await this._getGraphData();
        this._panel.webview.html = this._getHtmlForWebview(webview, graphData);
    }

    private async _getGraphData(): Promise<{ nodes: any[]; edges: any[] }> {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) {
            return { nodes: [], edges: [] };
        }

        const rootPath = workspaceFolders[0].uri.fsPath;
        const nodes: any[] = [];
        const edges: any[] = [];
        const moduleMap = new Map<string, any>();

        // Scan for modules
        try {
            const entries = fs.readdirSync(rootPath, { withFileTypes: true });

            for (const entry of entries) {
                if (!entry.isDirectory() || entry.name.startsWith('.')) continue;

                const moduleJsonPath = path.join(rootPath, entry.name, 'module.json');
                if (fs.existsSync(moduleJsonPath)) {
                    const content = fs.readFileSync(moduleJsonPath, 'utf-8');
                    const config = JSON.parse(content);

                    const node = {
                        id: config.name || entry.name,
                        language: config.language || 'unknown',
                        version: config.version || '0.0.0',
                        dependencies: config.dependencies || []
                    };

                    nodes.push(node);
                    moduleMap.set(node.id, node);
                }
            }

            // Build edges from dependencies
            for (const node of nodes) {
                for (const dep of node.dependencies) {
                    if (moduleMap.has(dep)) {
                        edges.push({
                            from: node.id,
                            to: dep
                        });
                    }
                }
            }
        } catch (error) {
            console.error('Failed to get graph data:', error);
        }

        return { nodes, edges };
    }

    private _openModule(moduleName: string): void {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) return;

        const moduleDir = path.join(workspaceFolders[0].uri.fsPath, moduleName);
        vscode.commands.executeCommand('revealInExplorer', vscode.Uri.file(moduleDir));
    }

    private _getHtmlForWebview(webview: vscode.Webview, data: { nodes: any[]; edges: any[] }): string {
        const languageColors: Record<string, string> = {
            'c': '#555555',
            'cpp': '#F34B7D',
            'c++': '#F34B7D',
            'csharp': '#178600',
            'c#': '#178600',
            'rust': '#DEA584',
            'go': '#00ADD8',
            'python': '#3572A5',
            'typescript': '#2B7489',
            'javascript': '#F1E05A',
            'java': '#B07219',
            'kotlin': '#A97BFF',
            'swift': '#FFAC45',
            'zig': '#EC915C',
            'nim': '#FFE953',
            'haskell': '#5E5086',
            'julia': '#A270BA',
            'lua': '#000080',
            'ruby': '#701516'
        };

        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Forge Dependency Graph</title>
    <style>
        body {
            margin: 0;
            padding: 20px;
            background: var(--vscode-editor-background);
            color: var(--vscode-editor-foreground);
            font-family: var(--vscode-font-family);
            overflow: hidden;
        }
        .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 20px;
        }
        h2 {
            margin: 0;
            color: var(--vscode-textLink-foreground);
        }
        button {
            background: var(--vscode-button-background);
            color: var(--vscode-button-foreground);
            border: none;
            padding: 6px 14px;
            cursor: pointer;
            border-radius: 2px;
        }
        button:hover {
            background: var(--vscode-button-hoverBackground);
        }
        #graph {
            width: 100%;
            height: calc(100vh - 100px);
            border: 1px solid var(--vscode-panel-border);
            border-radius: 4px;
        }
        svg {
            width: 100%;
            height: 100%;
        }
        .node {
            cursor: pointer;
        }
        .node circle {
            stroke: var(--vscode-editor-foreground);
            stroke-width: 2;
        }
        .node text {
            fill: var(--vscode-editor-foreground);
            font-size: 12px;
            text-anchor: middle;
            dominant-baseline: middle;
        }
        .edge {
            stroke: var(--vscode-editorWidget-border);
            stroke-width: 2;
            fill: none;
            marker-end: url(#arrowhead);
        }
        .legend {
            position: absolute;
            bottom: 30px;
            left: 30px;
            background: var(--vscode-editor-background);
            border: 1px solid var(--vscode-panel-border);
            padding: 10px;
            border-radius: 4px;
        }
        .legend-item {
            display: flex;
            align-items: center;
            margin: 4px 0;
            font-size: 11px;
        }
        .legend-color {
            width: 12px;
            height: 12px;
            border-radius: 50%;
            margin-right: 8px;
        }
        .empty-state {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            height: 100%;
            color: var(--vscode-descriptionForeground);
        }
    </style>
</head>
<body>
    <div class="header">
        <h2>Dependency Graph</h2>
        <button onclick="refresh()">↻ Refresh</button>
    </div>
    <div id="graph"></div>

    <script>
        const vscode = acquireVsCodeApi();
        const data = ${JSON.stringify(data)};
        const colors = ${JSON.stringify(languageColors)};

        function refresh() {
            vscode.postMessage({ command: 'refresh' });
        }

        function openModule(name) {
            vscode.postMessage({ command: 'openModule', module: name });
        }

        function render() {
            const container = document.getElementById('graph');
            
            if (data.nodes.length === 0) {
                container.innerHTML = '<div class="empty-state"><p>No modules found</p><p>Create modules to see the dependency graph</p></div>';
                return;
            }

            const width = container.clientWidth;
            const height = container.clientHeight;
            const centerX = width / 2;
            const centerY = height / 2;
            const radius = Math.min(width, height) * 0.35;

            // Calculate node positions (circular layout)
            const nodes = data.nodes.map((node, i) => {
                const angle = (2 * Math.PI * i) / data.nodes.length - Math.PI / 2;
                return {
                    ...node,
                    x: centerX + radius * Math.cos(angle),
                    y: centerY + radius * Math.sin(angle)
                };
            });

            const nodeMap = new Map(nodes.map(n => [n.id, n]));

            // Build SVG
            let svg = '<svg>';
            
            // Arrow marker
            svg += '<defs><marker id="arrowhead" markerWidth="10" markerHeight="7" refX="25" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="var(--vscode-editorWidget-border)" /></marker></defs>';

            // Draw edges
            for (const edge of data.edges) {
                const from = nodeMap.get(edge.from);
                const to = nodeMap.get(edge.to);
                if (from && to) {
                    svg += '<path class="edge" d="M' + from.x + ',' + from.y + ' L' + to.x + ',' + to.y + '" />';
                }
            }

            // Draw nodes
            for (const node of nodes) {
                const color = colors[node.language.toLowerCase()] || '#888888';
                svg += '<g class="node" onclick="openModule(\\'' + node.id + '\\')">';
                svg += '<circle cx="' + node.x + '" cy="' + node.y + '" r="20" fill="' + color + '" />';
                svg += '<text x="' + node.x + '" y="' + (node.y + 35) + '">' + node.id + '</text>';
                svg += '</g>';
            }

            svg += '</svg>';
            container.innerHTML = svg;

            // Add legend
            const usedLanguages = [...new Set(data.nodes.map(n => n.language))];
            if (usedLanguages.length > 0) {
                let legend = '<div class="legend">';
                for (const lang of usedLanguages) {
                    const color = colors[lang.toLowerCase()] || '#888888';
                    legend += '<div class="legend-item"><div class="legend-color" style="background:' + color + '"></div>' + lang + '</div>';
                }
                legend += '</div>';
                document.body.insertAdjacentHTML('beforeend', legend);
            }
        }

        render();
        window.addEventListener('resize', render);
    </script>
</body>
</html>`;
    }
}
