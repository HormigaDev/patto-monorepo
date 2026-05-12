import { stdin } from 'node:process';
export async function readCoreRequest(defaultCommand) {
    const raw = await readStdin();
    const parsed = raw.trim().length === 0 ? {} : JSON.parse(raw);
    const command = parsed.command ?? defaultCommand;
    if (command === undefined) {
        throw new Error('El JSON de stdin debe incluir "command".');
    }
    if (!isCoreCommand(command)) {
        throw new Error('El comando debe ser scan, lint, doctor o check.');
    }
    return {
        command,
        root: parsed.root ?? process.cwd(),
        lang: parsed.lang ?? 'auto',
    };
}
function readStdin() {
    return new Promise((resolve, reject) => {
        const chunks = [];
        stdin.on('data', (chunk) => chunks.push(chunk));
        stdin.on('error', reject);
        stdin.on('end', () => resolve(Buffer.concat(chunks).toString('utf8')));
        stdin.resume();
    });
}
function isCoreCommand(value) {
    return value === 'scan' || value === 'lint' || value === 'doctor' || value === 'check';
}
//# sourceMappingURL=stdin.js.map