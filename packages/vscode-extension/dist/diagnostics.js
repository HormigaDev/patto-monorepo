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
exports.applyDiagnostics = applyDiagnostics;
const node_fs_1 = require("node:fs");
const node_path_1 = __importDefault(require("node:path"));
const vscode = __importStar(require("vscode"));
function applyDiagnostics(collection, workspaceFolder, diagnostics) {
    collection.clear();
    const grouped = new Map();
    for (const diagnostic of diagnostics) {
        if (!diagnostic.file) {
            continue;
        }
        const absolutePath = node_path_1.default.resolve(workspaceFolder.uri.fsPath, diagnostic.file);
        const uri = vscode.Uri.file(absolutePath);
        const vscodeDiagnostic = new vscode.Diagnostic(rangeFromDiagnostic(diagnostic, absolutePath), diagnostic.hint ? `${diagnostic.message}\n${diagnostic.hint}` : diagnostic.message, severityFromLevel(diagnostic.level));
        vscodeDiagnostic.code = diagnostic.code;
        vscodeDiagnostic.source = 'patto';
        const key = uri.toString();
        grouped.set(key, [...(grouped.get(key) ?? []), vscodeDiagnostic]);
    }
    for (const [uri, items] of grouped.entries()) {
        collection.set(vscode.Uri.parse(uri), items);
    }
}
function rangeFromDiagnostic(diagnostic, absolutePath) {
    const line = Math.max(0, (diagnostic.line ?? 1) - 1);
    const character = Math.max(0, (diagnostic.column ?? 1) - 1);
    const width = inferTokenWidth(absolutePath, line, character);
    return new vscode.Range(line, character, line, character + width);
}
function inferTokenWidth(absolutePath, line, character) {
    if (!(0, node_fs_1.existsSync)(absolutePath)) {
        return 1;
    }
    const sourceLine = (0, node_fs_1.readFileSync)(absolutePath, 'utf8').split(/\r?\n/)[line];
    if (sourceLine === undefined) {
        return 1;
    }
    const match = /^[A-Za-z0-9_@$.-]+/.exec(sourceLine.slice(character));
    return Math.max(1, match?.[0].length ?? 1);
}
function severityFromLevel(level) {
    if (level === 'error') {
        return vscode.DiagnosticSeverity.Error;
    }
    if (level === 'warning') {
        return vscode.DiagnosticSeverity.Warning;
    }
    return vscode.DiagnosticSeverity.Information;
}
//# sourceMappingURL=diagnostics.js.map