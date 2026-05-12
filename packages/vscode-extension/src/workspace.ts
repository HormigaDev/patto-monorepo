import * as vscode from 'vscode';

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
