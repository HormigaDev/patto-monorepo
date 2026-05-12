import * as vscode from 'vscode';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { runProcess } from './process';
import type { PattoCoreCommand, PattoCoreEnvelope } from './types';

const CLI_PACKAGE = '@patto/cli';

export async function ensurePattoCli(): Promise<string | null> {
    const configuredPath = vscode.workspace.getConfiguration('patto').get<string>('cliPath')?.trim();

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

export async function showCliSetupMessage(): Promise<void> {
    const choice = await vscode.window.showWarningMessage(
        `Patto CLI no esta instalado. Instala ${CLI_PACKAGE} manualmente o configura una ruta personalizada.`,
        'Configurar ruta',
        'Abrir README',
    );

    if (choice === 'Configurar ruta') {
        await vscode.commands.executeCommand('workbench.action.openSettings', 'patto.cliPath');
    } else if (choice === 'Abrir README') {
        await vscode.env.openExternal(
            vscode.Uri.parse('https://github.com/HormigaDev/patto-monorepo/tree/main/packages/vscode-extension#cli-setup'),
        );
    }
}

export async function runPattoCore(
    cliPath: string,
    root: string,
    command: PattoCoreCommand,
): Promise<PattoCoreEnvelope> {
    const request = JSON.stringify({ command, root, lang: 'es' });
    const invocation = buildCliInvocation(cliPath, ['core', '--stdin']);
    const result = await runProcess(invocation.command, invocation.args, {
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

function buildCliInvocation(
    cliPath: string,
    args: string[],
): { readonly command: string; readonly args: string[] } {
    if (process.platform === 'win32' && cliPath.toLowerCase().endsWith('.cmd')) {
        return {
            command: 'cmd.exe',
            args: ['/d', '/s', '/c', cliPath, ...args],
        };
    }

    return { command: cliPath, args };
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

async function commandExists(command: string): Promise<boolean> {
    const checker = process.platform === 'win32' ? 'where' : 'which';
    const args = [command];

    try {
        const result = await runProcess(checker, args);
        return result.exitCode === 0;
    } catch {
        return false;
    }
}
