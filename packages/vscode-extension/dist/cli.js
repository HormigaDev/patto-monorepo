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
exports.runPattoCore = runPattoCore;
const vscode = __importStar(require("vscode"));
const node_fs_1 = require("node:fs");
const node_path_1 = __importDefault(require("node:path"));
const process_1 = require("./process");
const CLI_PACKAGE = '@patto/cli';
async function ensurePattoCli(output) {
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
    const autoInstall = vscode.workspace
        .getConfiguration('patto')
        .get('autoInstallCli', true);
    if (!autoInstall) {
        vscode.window.showWarningMessage('Patto CLI no esta instalado. Configura patto.cliPath o activa patto.autoInstallCli.');
        return null;
    }
    const nodeAvailable = await commandExists('node');
    if (!nodeAvailable) {
        vscode.window.showErrorMessage('Patto necesita Node.js instalado para usar el CLI.');
        return null;
    }
    const installed = await installCli(output);
    return installed ? 'patto' : null;
}
async function runPattoCore(cliPath, root, command) {
    const request = JSON.stringify({ command, root, lang: 'es' });
    const result = await (0, process_1.runProcess)(cliPath, ['core', '--stdin'], {
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
async function installCli(output) {
    const packageManager = (await commandExists('pnpm')) ? 'pnpm' : 'npm';
    if (packageManager === 'npm' && !(await commandExists('npm'))) {
        vscode.window.showErrorMessage('No encontre pnpm ni npm para instalar @patto/cli. Instala Node.js/npm y reintenta.');
        return false;
    }
    const args = packageManager === 'pnpm'
        ? ['add', '-g', CLI_PACKAGE]
        : ['install', '-g', CLI_PACKAGE];
    output.appendLine(`Instalando ${CLI_PACKAGE} con ${packageManager}...`);
    const choice = await vscode.window.showInformationMessage('Patto CLI no esta instalado. ¿Instalar @patto/cli globalmente?', 'Instalar', 'Cancelar');
    if (choice !== 'Instalar') {
        return false;
    }
    return vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: 'Instalando Patto CLI',
        cancellable: false,
    }, async () => {
        const result = await (0, process_1.runProcess)(packageManager, args);
        output.append(result.stdout);
        output.append(result.stderr);
        if (result.exitCode !== 0) {
            vscode.window.showErrorMessage(`No se pudo instalar ${CLI_PACKAGE}. Revisa la salida "Patto".`);
            return false;
        }
        if (!(await commandExists('patto'))) {
            vscode.window.showWarningMessage('Patto CLI se instalo, pero "patto" no aparece en PATH. Reinicia VSCode o configura patto.cliPath.');
            return false;
        }
        vscode.window.showInformationMessage('Patto CLI instalado correctamente.');
        return true;
    });
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