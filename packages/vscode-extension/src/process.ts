import { spawn } from 'node:child_process';

export interface PattoProcessResult {
    readonly exitCode: number;
    readonly stdout: string;
    readonly stderr: string;
}

export function runPattoCliProcess(
    cliCommand: string,
    args: string[],
    options: { readonly cwd?: string; readonly input?: string } = {},
): Promise<PattoProcessResult> {
    return new Promise((resolve, reject) => {
        const child = spawn(cliCommand, args, {
            cwd: options.cwd,
            windowsHide: true,
            stdio: ['pipe', 'pipe', 'pipe'],
        });
        const stdout: Buffer[] = [];
        const stderr: Buffer[] = [];

        child.stdout.on('data', (chunk: Buffer) => stdout.push(chunk));
        child.stderr.on('data', (chunk: Buffer) => stderr.push(chunk));
        child.on('error', reject);
        child.on('close', (code) => {
            resolve({
                exitCode: code ?? 1,
                stdout: Buffer.concat(stdout).toString('utf8'),
                stderr: Buffer.concat(stderr).toString('utf8'),
            });
        });

        if (options.input !== undefined) {
            child.stdin.end(options.input);
        } else {
            child.stdin.end();
        }
    });
}
