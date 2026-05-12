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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const node_fs_1 = require("node:fs");
const node_path_1 = __importDefault(require("node:path"));
const vscode = __importStar(require("vscode"));
const cli_1 = require("./cli");
const diagnostics_1 = require("./diagnostics");
const workspace_1 = require("./workspace");
let diagnosticTimer;
function activate(context) {
    const output = vscode.window.createOutputChannel('Patto');
    const collection = vscode.languages.createDiagnosticCollection('patto');
    context.subscriptions.push(output, collection);
    context.subscriptions.push(vscode.commands.registerCommand('patto.check', () => runDiagnostics('check', collection, output, true)), vscode.commands.registerCommand('patto.lint', () => runDiagnostics('lint', collection, output, true)), vscode.commands.registerCommand('patto.showCliSetup', async () => {
        await (0, cli_1.showCliSetupMessage)();
    }), vscode.workspace.onDidSaveTextDocument((document) => {
        const runOnSave = vscode.workspace
            .getConfiguration('patto')
            .get('runDiagnosticsOnSave', true);
        if (runOnSave && document.languageId === 'typescript') {
            scheduleDiagnostics(collection, output);
        }
    }), vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration('patto')) {
            scheduleDiagnostics(collection, output);
        }
    }));
    if (isPattoWorkspace()) {
        output.appendLine('Patto workspace detected. Diagnostics will run on save or by command.');
    }
}
function deactivate() {
    if (diagnosticTimer !== undefined) {
        clearTimeout(diagnosticTimer);
    }
}
function scheduleDiagnostics(collection, output) {
    if (!vscode.workspace.getConfiguration('patto').get('enableDiagnostics', true)) {
        collection.clear();
        return;
    }
    if (diagnosticTimer !== undefined) {
        clearTimeout(diagnosticTimer);
    }
    diagnosticTimer = setTimeout(() => {
        const command = vscode.workspace
            .getConfiguration('patto')
            .get('diagnosticsCommand', 'lint');
        void runDiagnostics(command, collection, output, false);
    }, 350);
}
async function runDiagnostics(command, collection, output, revealOutput) {
    const workspaceFolder = (0, workspace_1.getActiveWorkspaceFolder)();
    if (!workspaceFolder) {
        vscode.window.showWarningMessage('Abre un proyecto Patto para ejecutar diagnostics.');
        return;
    }
    const cliPath = await (0, cli_1.ensurePattoCli)();
    if (!cliPath) {
        return;
    }
    try {
        const envelope = await vscode.window.withProgress({
            location: vscode.ProgressLocation.Window,
            title: `Patto ${command}`,
        }, () => (0, cli_1.runPattoCore)(cliPath, workspaceFolder.uri.fsPath, command));
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
        vscode.window.showErrorMessage(`Patto fallo: ${message}`);
    }
}
function isPattoWorkspace() {
    const workspaceFolders = vscode.workspace.workspaceFolders ?? [];
    return workspaceFolders.some((folder) => {
        const root = folder.uri.fsPath;
        return ((0, node_fs_1.existsSync)(node_path_1.default.join(root, '.patto', 'config.json')) ||
            (0, node_fs_1.existsSync)(node_path_1.default.join(root, 'src', 'core', 'structures', 'BaseCommand.ts')));
    });
}
//# sourceMappingURL=extension.js.map