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
exports.ensurePattoCli = ensurePattoCli;
exports.showCliSetupMessage = showCliSetupMessage;
exports.runPattoCore = runPattoCore;
const vscode = __importStar(require("vscode"));
const node_fs_1 = require("node:fs");
const node_path_1 = __importDefault(require("node:path"));
const process_1 = require("./process");
const CLI_PACKAGE = '@patto/cli';
async function ensurePattoCli() {
    const configuredPath = vscode.workspace.getConfiguration('patto').get('cliPath')?.trim();
    if (configuredPath) {
        return configuredPath;
    }
    if (await commandExists('patto')) {
        return 'patto';
    }
    const localCli = findWorkspaceCli();
    if (localCli) {
        return localCli;
    }
    await showCliSetupMessage();
    return null;
}
async function showCliSetupMessage() {
    const choice = await vscode.window.showWarningMessage(`Patto CLI no esta instalado. Instala ${CLI_PACKAGE} manualmente o configura una ruta personalizada.`, 'Configurar ruta', 'Abrir README');
    if (choice === 'Configurar ruta') {
        await vscode.commands.executeCommand('workbench.action.openSettings', 'patto.cliPath');
    }
    else if (choice === 'Abrir README') {
        await vscode.env.openExternal(vscode.Uri.parse('https://github.com/HormigaDev/patto-monorepo/tree/main/packages/vscode-extension#cli-setup'));
    }
}
async function runPattoCore(cliPath, root, command) {
    const request = JSON.stringify({ command, root, lang: 'es' });
    const invocation = buildCliInvocation(cliPath, ['core', '--stdin']);
    const result = await (0, process_1.runProcess)(invocation.command, invocation.args, {
        cwd: root,
        input: request,
    });
    if (result.stderr.trim().length > 0) {
        console.warn(result.stderr);
    }
    try {
        return JSON.parse(result.stdout);
    }
    catch {
        throw new Error(`Patto CLI no devolvio JSON valido. stdout: ${result.stdout} stderr: ${result.stderr}`);
    }
}
function buildCliInvocation(cliPath, args) {
    if (process.platform === 'win32' && cliPath.toLowerCase().endsWith('.cmd')) {
        return {
            command: 'cmd.exe',
            args: ['/d', '/s', '/c', cliPath, ...args],
        };
    }
    return { command: cliPath, args };
}
function findWorkspaceCli() {
    const folders = vscode.workspace.workspaceFolders ?? [];
    const binary = process.platform === 'win32' ? 'patto.cmd' : 'patto';
    for (const folder of folders) {
        const candidate = node_path_1.default.join(folder.uri.fsPath, 'node_modules', '.bin', binary);
        if ((0, node_fs_1.existsSync)(candidate)) {
            return candidate;
        }
    }
    return null;
}
async function commandExists(command) {
    const checker = process.platform === 'win32' ? 'where' : 'which';
    const args = [command];
    try {
        const result = await (0, process_1.runProcess)(checker, args);
        return result.exitCode === 0;
    }
    catch {
        return false;
    }
}
//# sourceMappingURL=cli.js.map