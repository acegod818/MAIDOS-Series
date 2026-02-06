// MAIDOS-Forge VS Code Extension - Task Provider
// Code-QC v2.2B Compliant

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';

interface ForgeTaskDefinition extends vscode.TaskDefinition {
    task: 'build' | 'rebuild' | 'clean' | 'run' | 'watch';
    config?: 'Debug' | 'Release';
    module?: string;
}

export class ForgeTaskProvider implements vscode.TaskProvider {
    private tasks: vscode.Task[] | undefined;

    provideTasks(): vscode.Task[] | undefined {
        if (!this.tasks) {
            this.tasks = this.getDefaultTasks();
        }
        return this.tasks;
    }

    resolveTask(task: vscode.Task): vscode.Task | undefined {
        const definition = task.definition as ForgeTaskDefinition;
        
        if (definition.type === 'forge' && definition.task) {
            return this.createTask(definition);
        }

        return undefined;
    }

    private getDefaultTasks(): vscode.Task[] {
        const tasks: vscode.Task[] = [];

        // Build Debug
        tasks.push(this.createTask({
            type: 'forge',
            task: 'build',
            config: 'Debug'
        }));

        // Build Release
        tasks.push(this.createTask({
            type: 'forge',
            task: 'build',
            config: 'Release'
        }));

        // Rebuild
        tasks.push(this.createTask({
            type: 'forge',
            task: 'rebuild'
        }));

        // Clean
        tasks.push(this.createTask({
            type: 'forge',
            task: 'clean'
        }));

        // Run
        tasks.push(this.createTask({
            type: 'forge',
            task: 'run'
        }));

        // Watch
        tasks.push(this.createTask({
            type: 'forge',
            task: 'watch'
        }));

        // Per-module tasks
        const modules = this.getModules();
        for (const module of modules) {
            tasks.push(this.createTask({
                type: 'forge',
                task: 'build',
                module: module
            }));
        }

        return tasks;
    }

    private createTask(definition: ForgeTaskDefinition): vscode.Task {
        const args: string[] = [definition.task];

        if (definition.config) {
            args.push('-c', definition.config);
        }

        if (definition.module) {
            args.push('-m', definition.module);
        }

        const forgeExe = vscode.workspace.getConfiguration('forge').get('executablePath') || 'forge';
        const command = `${forgeExe} ${args.join(' ')}`;

        let taskName = `Forge: ${this.capitalizeFirst(definition.task)}`;
        if (definition.config) {
            taskName += ` (${definition.config})`;
        }
        if (definition.module) {
            taskName += ` - ${definition.module}`;
        }

        const execution = new vscode.ShellExecution(command);

        const task = new vscode.Task(
            definition,
            vscode.TaskScope.Workspace,
            taskName,
            'forge',
            execution,
            '$forge' // Problem matcher
        );

        // Set task group
        switch (definition.task) {
            case 'build':
            case 'rebuild':
                task.group = vscode.TaskGroup.Build;
                break;
            case 'clean':
                task.group = vscode.TaskGroup.Clean;
                break;
            case 'run':
                task.group = {
                    kind: vscode.TaskGroup.Test,
                    isDefault: false
                };
                break;
        }

        task.presentationOptions = {
            reveal: vscode.TaskRevealKind.Always,
            panel: vscode.TaskPanelKind.Shared,
            clear: true
        };

        return task;
    }

    private getModules(): string[] {
        const modules: string[] = [];
        const workspaceFolders = vscode.workspace.workspaceFolders;
        
        if (!workspaceFolders) return modules;

        const rootPath = workspaceFolders[0].uri.fsPath;
        
        try {
            const entries = fs.readdirSync(rootPath, { withFileTypes: true });
            
            for (const entry of entries) {
                if (!entry.isDirectory()) continue;
                if (entry.name.startsWith('.')) continue;

                const moduleJsonPath = path.join(rootPath, entry.name, 'module.json');
                if (fs.existsSync(moduleJsonPath)) {
                    modules.push(entry.name);
                }
            }
        } catch (error) {
            console.error('Failed to scan modules:', error);
        }

        return modules;
    }

    private capitalizeFirst(str: string): string {
        return str.charAt(0).toUpperCase() + str.slice(1);
    }
}
