import { spawn } from 'node:child_process';
import { resolveCoreBinary } from './resolve.js';
import type { CoreOutput, CoreRunOptions, CoreRunResult } from './types.js';

export async function runCore(options: CoreRunOptions): Promise<CoreRunResult> {
    const binary = resolveCoreBinary();
    const args = [options.command, '--json'];

    if (options.root !== undefined) {
        args.push('--root', options.root);
    }

    if (options.lang !== undefined) {
        args.push('--lang', options.lang);
    }

    const { exitCode, stdout, stderr } = await runProcess(binary, args);

    return {
        command: options.command,
        exitCode,
        stdout,
        stderr,
        output: parseCoreOutput(stdout),
    };
}

function runProcess(
    binary: string,
    args: string[],
): Promise<{ exitCode: number; stdout: string; stderr: string }> {
    return new Promise((resolve, reject) => {
        const child = spawn(binary, args, {
            stdio: ['ignore', 'pipe', 'pipe'],
            windowsHide: true,
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
    });
}

function parseCoreOutput(stdout: string): CoreOutput | null {
    const trimmed = stdout.trim();

    if (trimmed.length === 0) {
        return null;
    }

    try {
        return JSON.parse(trimmed) as CoreOutput;
    } catch {
        return null;
    }
}
