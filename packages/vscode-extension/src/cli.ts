import * as vscode from 'vscode';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { runProcess } from './process';
import type { PattoCoreCommand, PattoCoreEnvelope } from './types';

const CLI_PACKAGE = '@patto/cli';

export async function ensurePattoCli(output: vscode.OutputChannel): Promise<string | null> {
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

    const autoInstall = vscode.workspace
        .getConfiguration('patto')
        .get<boolean>('autoInstallCli', true);

    if (!autoInstall) {
        vscode.window.showWarningMessage(
            'Patto CLI no esta instalado. Configura patto.cliPath o activa patto.autoInstallCli.',
        );
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

export async function runPattoCore(
    cliPath: string,
    root: string,
    command: PattoCoreCommand,
): Promise<PattoCoreEnvelope> {
    const request = JSON.stringify({ command, root, lang: 'es' });
    const result = await runProcess(cliPath, ['core', '--stdin'], {
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

async function installCli(output: vscode.OutputChannel): Promise<boolean> {
    const packageManager = (await commandExists('pnpm')) ? 'pnpm' : 'npm';

    if (packageManager === 'npm' && !(await commandExists('npm'))) {
        vscode.window.showErrorMessage(
            'No encontre pnpm ni npm para instalar @patto/cli. Instala Node.js/npm y reintenta.',
        );
        return false;
    }

    const args =
        packageManager === 'pnpm'
            ? ['add', '-g', CLI_PACKAGE]
            : ['install', '-g', CLI_PACKAGE];

    output.appendLine(`Instalando ${CLI_PACKAGE} con ${packageManager}...`);

    const choice = await vscode.window.showInformationMessage(
        'Patto CLI no esta instalado. ¿Instalar @patto/cli globalmente?',
        'Instalar',
        'Cancelar',
    );

    if (choice !== 'Instalar') {
        return false;
    }

    return vscode.window.withProgress(
        {
            location: vscode.ProgressLocation.Notification,
            title: 'Instalando Patto CLI',
            cancellable: false,
        },
        async () => {
            const result = await runProcess(packageManager, args);
            output.append(result.stdout);
            output.append(result.stderr);

            if (result.exitCode !== 0) {
                vscode.window.showErrorMessage(
                    `No se pudo instalar ${CLI_PACKAGE}. Revisa la salida "Patto".`,
                );
                return false;
            }

            if (!(await commandExists('patto'))) {
                vscode.window.showWarningMessage(
                    'Patto CLI se instalo, pero "patto" no aparece en PATH. Reinicia VSCode o configura patto.cliPath.',
                );
                return false;
            }

            vscode.window.showInformationMessage('Patto CLI instalado correctamente.');
            return true;
        },
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
