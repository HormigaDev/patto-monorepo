import { spawn } from 'node:child_process';
import { resolveCoreBinary } from './resolve.js';
export async function runCore(options) {
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
function runProcess(binary, args) {
    return new Promise((resolve, reject) => {
        const child = spawn(binary, args, {
            stdio: ['ignore', 'pipe', 'pipe'],
            windowsHide: true,
        });
        const stdout = [];
        const stderr = [];
        child.stdout.on('data', (chunk) => stdout.push(chunk));
        child.stderr.on('data', (chunk) => stderr.push(chunk));
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
function parseCoreOutput(stdout) {
    const trimmed = stdout.trim();
    if (trimmed.length === 0) {
        return null;
    }
    try {
        return JSON.parse(trimmed);
    }
    catch {
        return null;
    }
}
//# sourceMappingURL=run.js.map