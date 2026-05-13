import * as vscode from 'vscode';
import { existsSync } from 'node:fs';
import path from 'node:path';

export function getActiveWorkspaceFolder(): vscode.WorkspaceFolder | null {
    const activeDocument = vscode.window.activeTextEditor?.document;

    if (activeDocument) {
        const folder = vscode.workspace.getWorkspaceFolder(activeDocument.uri);

        if (folder) {
            return folder;
        }
    }

    return vscode.workspace.workspaceFolders?.[0] ?? null;
}

export function isPattoDocument(document: vscode.TextDocument): boolean {
    const folder = vscode.workspace.getWorkspaceFolder(document.uri);

    if (!folder) {
        return false;
    }

    return document.uri.fsPath.includes(`${folder.uri.fsPath}`);
}

export function isPattoWorkspaceFolder(folder: vscode.WorkspaceFolder): boolean {
    const root = folder.uri.fsPath;

    return (
        existsSync(path.join(root, '.patto', 'config.json')) ||
        existsSync(path.join(root, 'src', 'core', 'structures', 'BaseCommand.ts'))
    );
}

export function getPattoWorkspaceFolders(): vscode.WorkspaceFolder[] {
    return (vscode.workspace.workspaceFolders ?? []).filter(isPattoWorkspaceFolder);
}

export function getActivePattoWorkspaceFolder(): vscode.WorkspaceFolder | null {
    const activeFolder = getActiveWorkspaceFolder();

    if (activeFolder && isPattoWorkspaceFolder(activeFolder)) {
        return activeFolder;
    }

    return getPattoWorkspaceFolders()[0] ?? null;
}
