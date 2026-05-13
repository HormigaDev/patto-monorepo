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
exports.resolvePattoCli = resolvePattoCli;
exports.runPattoCore = runPattoCore;
exports.showCliSetupMessage = showCliSetupMessage;
const vscode = __importStar(require("vscode"));
const node_fs_1 = require("node:fs");
const node_path_1 = __importDefault(require("node:path"));
const process_1 = require("./process");
const CLI_PACKAGE = '@patto/cli';
const CLI_COMMAND = 'patto';
async function resolvePattoCli() {
    const configuredPath = vscode.workspace.getConfiguration('patto').get('cliPath')?.trim();
    if (configuredPath) {
        return configuredPath;
    }
    const localCli = findWorkspaceCli();
    if (localCli) {
        return localCli;
    }
    return CLI_COMMAND;
}
async function runPattoCore(cliCommand, root, command) {
    const request = JSON.stringify({ command, root, lang: 'es' });
    const result = await (0, process_1.runPattoCliProcess)(cliCommand, ['core', '--stdin'], {
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
function showCliSetupMessage() {
    vscode.window.showWarningMessage(`Patto CLI no respondió. Instala ${CLI_PACKAGE}, asegúrate de que "patto" esté disponible en PATH o configura patto.cliPath.`);
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
//# sourceMappingURL=cli.js.map