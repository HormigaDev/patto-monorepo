import * as vscode from 'vscode';
import { resolvePattoCli, runPattoCore, showCliSetupMessage } from './cli';
import { applyDiagnostics } from './diagnostics';
import type { PattoCoreCommand } from './types';
import {
    getActivePattoWorkspaceFolder,
    getPattoWorkspaceFolders,
    isPattoWorkspaceFolder,
} from './workspace';

let diagnosticTimer: NodeJS.Timeout | undefined;
let scheduledFolder: vscode.WorkspaceFolder | undefined;
let isRunningDiagnostics = false;
let shouldRunAgain = false;

export function activate(context: vscode.ExtensionContext): void {
    const output = vscode.window.createOutputChannel('Patto');
    const collection = vscode.languages.createDiagnosticCollection('patto');

    context.subscriptions.push(output, collection);

    const pattoFolders = getPattoWorkspaceFolders();

    if (pattoFolders.length === 0) {
        output.appendLine('No Patto workspace detected. Patto extension is inactive for this workspace.');
        return;
    }

    output.appendLine(`Patto workspace detected: ${pattoFolders.map((folder) => folder.name).join(', ')}`);

    context.subscriptions.push(
        vscode.commands.registerCommand('patto.check', () =>
            runDiagnostics('check', collection, output, true),
        ),
        vscode.commands.registerCommand('patto.lint', () =>
            runDiagnostics('lint', collection, output, true),
        ),
        vscode.commands.registerCommand('patto.showCliSetup', async () => {
            showCliSetupMessage();
        }),
        vscode.workspace.onDidSaveTextDocument((document) => {
            if (shouldReactToDocument(document, 'runDiagnosticsOnSave')) {
                scheduleDiagnostics(collection, output, 'save', workspaceFolderForDocument(document));
            }
        }),
        vscode.workspace.onDidChangeTextDocument((event) => {
            if (shouldReactToDocument(event.document, 'runDiagnosticsOnChange')) {
                scheduleDiagnostics(
                    collection,
                    output,
                    'change',
                    workspaceFolderForDocument(event.document),
                );
            }
        }),
        vscode.workspace.onDidChangeConfiguration((event) => {
            if (event.affectsConfiguration('patto')) {
                scheduleDiagnostics(collection, output, 'config', getActivePattoWorkspaceFolder() ?? undefined);
            }
        }),
    );

    const watcher = vscode.workspace.createFileSystemWatcher(
        '**/{package.json,tsconfig.json,.patto/config.json,.patto/config.schema.json,src/**/*.ts,src/i18n/locale/**/*.ts}',
    );

    context.subscriptions.push(
        watcher,
        watcher.onDidCreate((uri) => scheduleDiagnostics(collection, output, 'file-create', workspaceFolderForUri(uri))),
        watcher.onDidChange((uri) => scheduleDiagnostics(collection, output, 'file-change', workspaceFolderForUri(uri))),
        watcher.onDidDelete((uri) => scheduleDiagnostics(collection, output, 'file-delete', workspaceFolderForUri(uri))),
    );

    if (vscode.workspace.getConfiguration('patto').get<boolean>('runDiagnosticsOnOpen', true)) {
        scheduleDiagnostics(collection, output, 'open', pattoFolders[0]);
    }
}

export function deactivate(): void {
    if (diagnosticTimer !== undefined) {
        clearTimeout(diagnosticTimer);
    }
}

function scheduleDiagnostics(
    collection: vscode.DiagnosticCollection,
    output: vscode.OutputChannel,
    reason: string,
    folder?: vscode.WorkspaceFolder,
): void {
    if (!vscode.workspace.getConfiguration('patto').get<boolean>('enableDiagnostics', true)) {
        collection.clear();
        return;
    }

    scheduledFolder = folder ?? scheduledFolder ?? getActivePattoWorkspaceFolder() ?? undefined;

    if (!scheduledFolder || !isPattoWorkspaceFolder(scheduledFolder)) {
        collection.clear();
        return;
    }

    if (diagnosticTimer !== undefined) {
        clearTimeout(diagnosticTimer);
    }

    const debounceMs = vscode.workspace
        .getConfiguration('patto')
        .get<number>('diagnosticsDebounceMs', 800);

    diagnosticTimer = setTimeout(() => {
        const command = vscode.workspace
            .getConfiguration('patto')
            .get<PattoCoreCommand>('diagnosticsCommand', 'check');
        output.appendLine(`Scheduling Patto ${command} (${reason}).`);
        void runDiagnostics(command, collection, output, false, scheduledFolder);
    }, debounceMs);
}

async function runDiagnostics(
    command: PattoCoreCommand,
    collection: vscode.DiagnosticCollection,
    output: vscode.OutputChannel,
    revealOutput: boolean,
    folder?: vscode.WorkspaceFolder,
): Promise<void> {
    const workspaceFolder = folder ?? getActivePattoWorkspaceFolder();

    if (!workspaceFolder) {
        if (revealOutput) {
            vscode.window.showWarningMessage('Este workspace no parece ser un proyecto Patto.');
        }
        return;
    }

    if (!isPattoWorkspaceFolder(workspaceFolder)) {
        collection.clear();
        if (revealOutput) {
            vscode.window.showWarningMessage('Patto esta inactivo: el workspace no es un proyecto Patto.');
        }
        return;
    }

    if (isRunningDiagnostics) {
        shouldRunAgain = true;
        scheduledFolder = workspaceFolder;
        return;
    }

    const cliCommand = await resolvePattoCli();

    if (!cliCommand) {
        showCliSetupMessage();
        return;
    }

    try {
        isRunningDiagnostics = true;
        const envelope = await vscode.window.withProgress(
            {
                location: vscode.ProgressLocation.Window,
                title: `Patto ${command}`,
            },
            () => runPattoCore(cliCommand, workspaceFolder.uri.fsPath, command),
        );

        applyDiagnostics(collection, workspaceFolder, envelope.diagnostics);
        output.appendLine(
            `Patto ${command}: ${envelope.diagnostics.length} diagnostic(s), exit ${envelope.exitCode}.`,
        );

        if (envelope.stderr.trim().length > 0) {
            output.appendLine(envelope.stderr);
        }

        if (revealOutput) {
            output.show(true);
        }
    } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        output.appendLine(message);
        output.show(true);
        showCliSetupMessage();
    } finally {
        isRunningDiagnostics = false;

        if (shouldRunAgain) {
            shouldRunAgain = false;
            scheduleDiagnostics(collection, output, 'queued', scheduledFolder);
        }
    }
}

function shouldReactToDocument(
    document: vscode.TextDocument,
    setting: 'runDiagnosticsOnChange' | 'runDiagnosticsOnSave',
): boolean {
    if (!vscode.workspace.getConfiguration('patto').get<boolean>(setting, true)) {
        return false;
    }

    if (document.uri.scheme !== 'file') {
        return false;
    }

    const folder = workspaceFolderForDocument(document);

    if (!folder || !isPattoWorkspaceFolder(folder)) {
        return false;
    }

    return (
        document.languageId === 'typescript' ||
        document.languageId === 'json' ||
        document.fileName.endsWith('.env') ||
        document.fileName.endsWith('.command.ts') ||
        document.fileName.endsWith('.plugin.ts') ||
        document.fileName.includes(`${folder.uri.fsPath}${pathSeparator()}src${pathSeparator()}i18n${pathSeparator()}`)
    );
}

function workspaceFolderForDocument(document: vscode.TextDocument): vscode.WorkspaceFolder | undefined {
    return workspaceFolderForUri(document.uri);
}

function pathSeparator(): string {
    return process.platform === 'win32' ? '\\' : '/';
}

function workspaceFolderForUri(uri: vscode.Uri): vscode.WorkspaceFolder | undefined {
    return vscode.workspace.getWorkspaceFolder(uri);
}
