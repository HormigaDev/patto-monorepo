import * as vscode from 'vscode';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { runPattoCliProcess } from './process';
import type { PattoCoreCommand, PattoCoreEnvelope } from './types';

const CLI_PACKAGE = '@patto/cli';
const CLI_COMMAND = 'patto';

export async function resolvePattoCli(): Promise<string | null> {
    const configuredPath = vscode.workspace.getConfiguration('patto').get<string>('cliPath')?.trim();

    if (configuredPath) {
        return configuredPath;
    }

    const localCli = findWorkspaceCli();

    if (localCli) {
        return localCli;
    }

    return CLI_COMMAND;
}

export async function runPattoCore(
    cliCommand: string,
    root: string,
    command: PattoCoreCommand,
): Promise<PattoCoreEnvelope> {
    const request = JSON.stringify({ command, root, lang: 'es' });
    const result = await runPattoCliProcess(cliCommand, ['core', '--stdin'], {
        cwd: root,
        input: request,
    });

    if (result.stderr.trim().length > 0) {
        console.warn(result.stderr);
    }

    try {
        return JSON.parse(result.stdout) as PattoCoreEnvelope;
    } catch {
        throw new Error(
            `Patto CLI no devolvio JSON valido. stdout: ${result.stdout} stderr: ${result.stderr}`,
        );
    }
}

export function showCliSetupMessage(): void {
    vscode.window.showWarningMessage(
        `Patto CLI no respondió. Instala ${CLI_PACKAGE}, asegúrate de que "patto" esté disponible en PATH o configura patto.cliPath.`,
    );
}

function findWorkspaceCli(): string | null {
    const folders = vscode.workspace.workspaceFolders ?? [];
    const binary = process.platform === 'win32' ? 'patto.cmd' : 'patto';

    for (const folder of folders) {
        const candidate = path.join(folder.uri.fsPath, 'node_modules', '.bin', binary);

        if (existsSync(candidate)) {
            return candidate;
        }
    }

    return null;
}
