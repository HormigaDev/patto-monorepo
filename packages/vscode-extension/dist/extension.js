"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const cli_1 = require("./cli");
const diagnostics_1 = require("./diagnostics");
const workspace_1 = require("./workspace");
let diagnosticTimer;
let scheduledFolder;
let isRunningDiagnostics = false;
let shouldRunAgain = false;
function activate(context) {
    const output = vscode.window.createOutputChannel('Patto');
    const collection = vscode.languages.createDiagnosticCollection('patto');
    context.subscriptions.push(output, collection);
    const pattoFolders = (0, workspace_1.getPattoWorkspaceFolders)();
    if (pattoFolders.length === 0) {
        output.appendLine('No Patto workspace detected. Patto extension is inactive for this workspace.');
        return;
    }
    output.appendLine(`Patto workspace detected: ${pattoFolders.map((folder) => folder.name).join(', ')}`);
    context.subscriptions.push(vscode.commands.registerCommand('patto.check', () => runDiagnostics('check', collection, output, true)), vscode.commands.registerCommand('patto.lint', () => runDiagnostics('lint', collection, output, true)), vscode.commands.registerCommand('patto.showCliSetup', async () => {
        (0, cli_1.showCliSetupMessage)();
    }), vscode.workspace.onDidSaveTextDocument((document) => {
        if (shouldReactToDocument(document, 'runDiagnosticsOnSave')) {
            scheduleDiagnostics(collection, output, 'save', workspaceFolderForDocument(document));
        }
    }), vscode.workspace.onDidChangeTextDocument((event) => {
        if (shouldReactToDocument(event.document, 'runDiagnosticsOnChange')) {
            scheduleDiagnostics(collection, output, 'change', workspaceFolderForDocument(event.document));
        }
    }), vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration('patto')) {
            scheduleDiagnostics(collection, output, 'config', (0, workspace_1.getActivePattoWorkspaceFolder)() ?? undefined);
        }
    }));
    const watcher = vscode.workspace.createFileSystemWatcher('**/{package.json,tsconfig.json,.patto/config.json,src/**/*.ts}');
    context.subscriptions.push(watcher, watcher.onDidCreate((uri) => scheduleDiagnostics(collection, output, 'file-create', workspaceFolderForUri(uri))), watcher.onDidChange((uri) => scheduleDiagnostics(collection, output, 'file-change', workspaceFolderForUri(uri))), watcher.onDidDelete((uri) => scheduleDiagnostics(collection, output, 'file-delete', workspaceFolderForUri(uri))));
    if (vscode.workspace.getConfiguration('patto').get('runDiagnosticsOnOpen', true)) {
        scheduleDiagnostics(collection, output, 'open', pattoFolders[0]);
    }
}
function deactivate() {
    if (diagnosticTimer !== undefined) {
        clearTimeout(diagnosticTimer);
    }
}
function scheduleDiagnostics(collection, output, reason, folder) {
    if (!vscode.workspace.getConfiguration('patto').get('enableDiagnostics', true)) {
        collection.clear();
        return;
    }
    scheduledFolder = folder ?? scheduledFolder ?? (0, workspace_1.getActivePattoWorkspaceFolder)() ?? undefined;
    if (!scheduledFolder || !(0, workspace_1.isPattoWorkspaceFolder)(scheduledFolder)) {
        collection.clear();
        return;
    }
    if (diagnosticTimer !== undefined) {
        clearTimeout(diagnosticTimer);
    }
    const debounceMs = vscode.workspace
        .getConfiguration('patto')
        .get('diagnosticsDebounceMs', 800);
    diagnosticTimer = setTimeout(() => {
        const command = vscode.workspace
            .getConfiguration('patto')
            .get('diagnosticsCommand', 'check');
        output.appendLine(`Scheduling Patto ${command} (${reason}).`);
        void runDiagnostics(command, collection, output, false, scheduledFolder);
    }, debounceMs);
}
async function runDiagnostics(command, collection, output, revealOutput, folder) {
    const workspaceFolder = folder ?? (0, workspace_1.getActivePattoWorkspaceFolder)();
    if (!workspaceFolder) {
        if (revealOutput) {
            vscode.window.showWarningMessage('Este workspace no parece ser un proyecto Patto.');
        }
        return;
    }
    if (!(0, workspace_1.isPattoWorkspaceFolder)(workspaceFolder)) {
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
    const cliCommand = await (0, cli_1.resolvePattoCli)();
    if (!cliCommand) {
        (0, cli_1.showCliSetupMessage)();
        return;
    }
    try {
        isRunningDiagnostics = true;
        const envelope = await vscode.window.withProgress({
            location: vscode.ProgressLocation.Window,
            title: `Patto ${command}`,
        }, () => (0, cli_1.runPattoCore)(cliCommand, workspaceFolder.uri.fsPath, command));
        (0, diagnostics_1.applyDiagnostics)(collection, workspaceFolder, envelope.diagnostics);
        output.appendLine(`Patto ${command}: ${envelope.diagnostics.length} diagnostic(s), exit ${envelope.exitCode}.`);
        if (envelope.stderr.trim().length > 0) {
            output.appendLine(envelope.stderr);
        }
        if (revealOutput) {
            output.show(true);
        }
    }
    catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        output.appendLine(message);
        output.show(true);
        (0, cli_1.showCliSetupMessage)();
    }
    finally {
        isRunningDiagnostics = false;
        if (shouldRunAgain) {
            shouldRunAgain = false;
            scheduleDiagnostics(collection, output, 'queued', scheduledFolder);
        }
    }
}
function shouldReactToDocument(document, setting) {
    if (!vscode.workspace.getConfiguration('patto').get(setting, true)) {
        return false;
    }
    if (document.uri.scheme !== 'file') {
        return false;
    }
    const folder = workspaceFolderForDocument(document);
    if (!folder || !(0, workspace_1.isPattoWorkspaceFolder)(folder)) {
        return false;
    }
    return (document.languageId === 'typescript' ||
        document.languageId === 'json' ||
        document.fileName.endsWith('.env') ||
        document.fileName.endsWith('.command.ts') ||
        document.fileName.endsWith('.plugin.ts'));
}
function workspaceFolderForDocument(document) {
    return workspaceFolderForUri(document.uri);
}
function workspaceFolderForUri(uri) {
    return vscode.workspace.getWorkspaceFolder(uri);
}
//# sourceMappingURL=extension.js.map