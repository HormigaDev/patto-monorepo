import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import * as vscode from 'vscode';
import type { PattoDiagnostic } from './types';

export function applyDiagnostics(
    collection: vscode.DiagnosticCollection,
    workspaceFolder: vscode.WorkspaceFolder,
    diagnostics: readonly PattoDiagnostic[],
): void {
    collection.clear();

    const grouped = new Map<string, vscode.Diagnostic[]>();

    for (const diagnostic of diagnostics) {
        if (!diagnostic.file) {
            continue;
        }

        const absolutePath = path.resolve(workspaceFolder.uri.fsPath, diagnostic.file);
        const uri = vscode.Uri.file(absolutePath);
        const vscodeDiagnostic = new vscode.Diagnostic(
            rangeFromDiagnostic(diagnostic, absolutePath),
            diagnostic.hint ? `${diagnostic.message}\n${diagnostic.hint}` : diagnostic.message,
            severityFromLevel(diagnostic.level),
        );

        vscodeDiagnostic.code = diagnostic.code;
        vscodeDiagnostic.source = 'patto';

        const key = uri.toString();
        grouped.set(key, [...(grouped.get(key) ?? []), vscodeDiagnostic]);
    }

    for (const [uri, items] of grouped.entries()) {
        collection.set(vscode.Uri.parse(uri), items);
    }
}

function rangeFromDiagnostic(diagnostic: PattoDiagnostic, absolutePath: string): vscode.Range {
    const line = Math.max(0, (diagnostic.line ?? 1) - 1);
    const character = Math.max(0, (diagnostic.column ?? 1) - 1);
    const width = inferTokenWidth(absolutePath, line, character);

    return new vscode.Range(line, character, line, character + width);
}

function inferTokenWidth(absolutePath: string, line: number, character: number): number {
    if (!existsSync(absolutePath)) {
        return 1;
    }

    const sourceLine = readFileSync(absolutePath, 'utf8').split(/\r?\n/)[line];

    if (sourceLine === undefined) {
        return 1;
    }

    const match = /^[A-Za-z0-9_@$.-]+/.exec(sourceLine.slice(character));
    return Math.max(1, match?.[0].length ?? 1);
}

function severityFromLevel(level: PattoDiagnostic['level']): vscode.DiagnosticSeverity {
    if (level === 'error') {
        return vscode.DiagnosticSeverity.Error;
    }

    if (level === 'warning') {
        return vscode.DiagnosticSeverity.Warning;
    }

    return vscode.DiagnosticSeverity.Information;
}
