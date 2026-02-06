// MAIDOS-Forge VS Code Extension
// Code-QC v2.2B Compliant

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { ForgeTaskProvider } from './taskProvider';
import { ForgeModulesProvider } from './modulesProvider';
import { ForgeToolchainsProvider } from './toolchainsProvider';
import { ForgeDiagnostics } from './diagnostics';
import { ForgeCommands } from './commands';
import { ForgeGraphPanel } from './graphPanel';
import { ForgeFfiPanel } from './ffiPanel';

let outputChannel: vscode.OutputChannel;
let diagnostics: ForgeDiagnostics;
let commands: ForgeCommands;
let modulesProvider: ForgeModulesProvider;
let toolchainsProvider: ForgeToolchainsProvider;

export async function activate(context: vscode.ExtensionContext) {
    console.log('Forge extension activating...');

    // Create output channel
    outputChannel = vscode.window.createOutputChannel('Forge');
    context.subscriptions.push(outputChannel);

    // Initialize diagnostics
    diagnostics = new ForgeDiagnostics();
    context.subscriptions.push(diagnostics);

    // Initialize commands
    commands = new ForgeCommands(outputChannel, diagnostics);

    // Check for forge.json in workspace
    const forgeConfigExists = await checkForgeProject();
    vscode.commands.executeCommand('setContext', 'forge.projectLoaded', forgeConfigExists);

    // Register task provider
    const taskProvider = new ForgeTaskProvider();
    context.subscriptions.push(
        vscode.tasks.registerTaskProvider('forge', taskProvider)
    );

    // Register tree views
    modulesProvider = new ForgeModulesProvider();
    toolchainsProvider = new ForgeToolchainsProvider();

    context.subscriptions.push(
        vscode.window.registerTreeDataProvider('forgeModules', modulesProvider),
        vscode.window.registerTreeDataProvider('forgeToolchains', toolchainsProvider)
    );

    // Register commands
    registerCommands(context);

    // Watch for forge.json changes
    const watcher = vscode.workspace.createFileSystemWatcher('**/forge.json');
    watcher.onDidChange(() => refreshProject());
    watcher.onDidCreate(() => refreshProject());
    watcher.onDidDelete(() => refreshProject());
    context.subscriptions.push(watcher);

    // Build on save (if enabled)
    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument(async (document) => {
            const config = vscode.workspace.getConfiguration('forge');
            if (config.get('buildOnSave')) {
                await commands.build();
            }
        })
    );

    // Auto-watch (if enabled)
    const config = vscode.workspace.getConfiguration('forge');
    if (config.get('autoWatch') && forgeConfigExists) {
        commands.watch();
    }

    outputChannel.appendLine('Forge extension activated');
    
    if (forgeConfigExists) {
        outputChannel.appendLine('Forge project detected');
        await refreshProject();
    }
}

export function deactivate() {
    outputChannel?.appendLine('Forge extension deactivated');
}

async function checkForgeProject(): Promise<boolean> {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders) return false;

    for (const folder of workspaceFolders) {
        const forgeJson = path.join(folder.uri.fsPath, 'forge.json');
        if (fs.existsSync(forgeJson)) {
            return true;
        }
    }
    return false;
}

async function refreshProject() {
    const exists = await checkForgeProject();
    vscode.commands.executeCommand('setContext', 'forge.projectLoaded', exists);
    
    if (exists) {
        modulesProvider.refresh();
        toolchainsProvider.refresh();
    }
}

function registerCommands(context: vscode.ExtensionContext) {
    // Basic commands
    context.subscriptions.push(
        vscode.commands.registerCommand('forge.init', () => commands.init()),
        vscode.commands.registerCommand('forge.build', () => commands.build()),
        vscode.commands.registerCommand('forge.rebuild', () => commands.rebuild()),
        vscode.commands.registerCommand('forge.clean', () => commands.clean()),
        vscode.commands.registerCommand('forge.run', () => commands.run()),
        vscode.commands.registerCommand('forge.watch', () => commands.watch()),
        vscode.commands.registerCommand('forge.addModule', () => commands.addModule()),
        vscode.commands.registerCommand('forge.checkToolchain', () => commands.checkToolchain())
    );

    // Panel commands
    context.subscriptions.push(
        vscode.commands.registerCommand('forge.showGraph', () => {
            ForgeGraphPanel.createOrShow(context.extensionUri);
        }),
        vscode.commands.registerCommand('forge.showFfi', () => {
            ForgeFfiPanel.createOrShow(context.extensionUri);
        })
    );

    // Studio command
    context.subscriptions.push(
        vscode.commands.registerCommand('forge.openStudio', () => commands.openStudio())
    );

    // Tree view commands
    context.subscriptions.push(
        vscode.commands.registerCommand('forge.refreshModules', () => modulesProvider.refresh()),
        vscode.commands.registerCommand('forge.refreshToolchains', () => toolchainsProvider.refresh()),
        vscode.commands.registerCommand('forge.buildModule', (node) => commands.buildModule(node.id))
    );
}
