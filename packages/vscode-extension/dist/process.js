"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.runProcess = runProcess;
const node_child_process_1 = require("node:child_process");
function runProcess(command, args, options = {}) {
    return new Promise((resolve, reject) => {
        const child = (0, node_child_process_1.spawn)(command, args, {
            cwd: options.cwd,
            windowsHide: true,
            stdio: ['pipe', 'pipe', 'pipe'],
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
        if (options.input !== undefined) {
            child.stdin.end(options.input);
        }
        else {
            child.stdin.end();
        }
    });
}
//# sourceMappingURL=process.js.map