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
exports.getActiveWorkspaceFolder = getActiveWorkspaceFolder;
exports.isPattoDocument = isPattoDocument;
exports.isPattoWorkspaceFolder = isPattoWorkspaceFolder;
exports.getPattoWorkspaceFolders = getPattoWorkspaceFolders;
exports.getActivePattoWorkspaceFolder = getActivePattoWorkspaceFolder;
const vscode = __importStar(require("vscode"));
const node_fs_1 = require("node:fs");
const node_path_1 = __importDefault(require("node:path"));
function getActiveWorkspaceFolder() {
    const activeDocument = vscode.window.activeTextEditor?.document;
    if (activeDocument) {
        const folder = vscode.workspace.getWorkspaceFolder(activeDocument.uri);
        if (folder) {
            return folder;
        }
    }
    return vscode.workspace.workspaceFolders?.[0] ?? null;
}
function isPattoDocument(document) {
    const folder = vscode.workspace.getWorkspaceFolder(document.uri);
    if (!folder) {
        return false;
    }
    return document.uri.fsPath.includes(`${folder.uri.fsPath}`);
}
function isPattoWorkspaceFolder(folder) {
    const root = folder.uri.fsPath;
    return ((0, node_fs_1.existsSync)(node_path_1.default.join(root, '.patto', 'config.json')) ||
        (0, node_fs_1.existsSync)(node_path_1.default.join(root, 'src', 'core', 'structures', 'BaseCommand.ts')));
}
function getPattoWorkspaceFolders() {
    return (vscode.workspace.workspaceFolders ?? []).filter(isPattoWorkspaceFolder);
}
function getActivePattoWorkspaceFolder() {
    const activeFolder = getActiveWorkspaceFolder();
    if (activeFolder && isPattoWorkspaceFolder(activeFolder)) {
        return activeFolder;
    }
    return getPattoWorkspaceFolders()[0] ?? null;
}
//# sourceMappingURL=workspace.js.map