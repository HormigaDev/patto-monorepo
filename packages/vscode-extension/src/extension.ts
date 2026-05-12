import { existsSync } from 'node:fs';
import path from 'node:path';
import * as vscode from 'vscode';
import { ensurePattoCli, runPattoCore, showCliSetupMessage } from './cli';
import { applyDiagnostics } from './diagnostics';
import type { PattoCoreCommand } from './types';
import { getActiveWorkspaceFolder } from './workspace';

let diagnosticTimer: NodeJS.Timeout | undefined;

export function activate(context: vscode.ExtensionContext): void {
    const output = vscode.window.createOutputChannel('Patto');
    const collection = vscode.languages.createDiagnosticCollection('patto');

    context.subscriptions.push(output, collection);
    context.subscriptions.push(
        vscode.commands.registerCommand('patto.check', () =>
            runDiagnostics('check', collection, output, true),
        ),
        vscode.commands.registerCommand('patto.lint', () =>
            runDiagnostics('lint', collection, output, true),
        ),
        vscode.commands.registerCommand('patto.showCliSetup', async () => {
            await showCliSetupMessage();
        }),
        vscode.workspace.onDidSaveTextDocument((document) => {
            const runOnSave = vscode.workspace
                .getConfiguration('patto')
                .get<boolean>('runDiagnosticsOnSave', true);

            if (runOnSave && document.languageId === 'typescript') {
                scheduleDiagnostics(collection, output);
            }
        }),
        vscode.workspace.onDidChangeConfiguration((event) => {
            if (event.affectsConfiguration('patto')) {
                scheduleDiagnostics(collection, output);
            }
        }),
    );

    if (isPattoWorkspace()) {
        output.appendLine('Patto workspace detected. Diagnostics will run on save or by command.');
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
): void {
    if (!vscode.workspace.getConfiguration('patto').get<boolean>('enableDiagnostics', true)) {
        collection.clear();
        return;
    }

    if (diagnosticTimer !== undefined) {
        clearTimeout(diagnosticTimer);
    }

    diagnosticTimer = setTimeout(() => {
        const command = vscode.workspace
            .getConfiguration('patto')
            .get<PattoCoreCommand>('diagnosticsCommand', 'lint');
        void runDiagnostics(command, collection, output, false);
    }, 350);
}

async function runDiagnostics(
    command: PattoCoreCommand,
    collection: vscode.DiagnosticCollection,
    output: vscode.OutputChannel,
    revealOutput: boolean,
): Promise<void> {
    const workspaceFolder = getActiveWorkspaceFolder();

    if (!workspaceFolder) {
        vscode.window.showWarningMessage('Abre un proyecto Patto para ejecutar diagnostics.');
        return;
    }

    const cliPath = await ensurePattoCli();

    if (!cliPath) {
        return;
    }

    try {
        const envelope = await vscode.window.withProgress(
            {
                location: vscode.ProgressLocation.Window,
                title: `Patto ${command}`,
            },
            () => runPattoCore(cliPath, workspaceFolder.uri.fsPath, command),
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
        vscode.window.showErrorMessage(`Patto fallo: ${message}`);
    }
}

function isPattoWorkspace(): boolean {
    const workspaceFolders = vscode.workspace.workspaceFolders ?? [];

    return workspaceFolders.some((folder) => {
        const root = folder.uri.fsPath;
        return (
            existsSync(path.join(root, '.patto', 'config.json')) ||
            existsSync(path.join(root, 'src', 'core', 'structures', 'BaseCommand.ts'))
        );
    });
}
