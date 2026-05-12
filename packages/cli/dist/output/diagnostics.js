import chalk from 'chalk';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
export function printHumanOutput(output, options) {
    const diagnostics = output?.diagnostics ?? [];
    if (diagnostics.length === 0) {
        console.log(chalk.green(`patto ${options.command}: sin diagnosticos.`));
        printSummary(output);
        return;
    }
    for (const diagnostic of diagnostics) {
        printDiagnostic(diagnostic, options.root);
    }
    printSummary(output);
}
export function diagnosticsFromOutput(output) {
    return output?.diagnostics ?? [];
}
function printDiagnostic(diagnostic, root) {
    const color = colorForLevel(diagnostic.level);
    const level = color(diagnostic.level.toUpperCase());
    const location = formatLocation(diagnostic, root);
    console.log(`${location} ${level} ${chalk.dim(diagnostic.code)}`);
    console.log(`  ${diagnostic.message}`);
    if (diagnostic.file && diagnostic.line) {
        printSourceFrame(root, diagnostic);
    }
    if (diagnostic.hint) {
        console.log(chalk.dim(`  hint: ${diagnostic.hint}`));
    }
    console.log('');
}
function printSourceFrame(root, diagnostic) {
    const filePath = path.resolve(root, diagnostic.file ?? '');
    if (!existsSync(filePath)) {
        return;
    }
    const lines = readFileSync(filePath, 'utf8').split(/\r?\n/);
    const lineNumber = diagnostic.line ?? 1;
    const sourceLine = lines[lineNumber - 1];
    if (sourceLine === undefined) {
        return;
    }
    const gutter = String(lineNumber).padStart(4, ' ');
    const column = Math.max(1, diagnostic.column ?? 1);
    const underlineStart = Math.max(0, column - 1);
    const underlineWidth = inferUnderlineWidth(sourceLine, underlineStart);
    const underline = `${' '.repeat(underlineStart)}${'^'.repeat(underlineWidth)}`;
    console.log(chalk.dim(`${gutter} | `) + sourceLine);
    console.log(chalk.dim('     | ') + colorForLevel(diagnostic.level)(underline));
}
function inferUnderlineWidth(sourceLine, start) {
    const rest = sourceLine.slice(start);
    const match = /^[A-Za-z0-9_@$.-]+/.exec(rest.trimStart());
    const leadingSpaces = rest.length - rest.trimStart().length;
    if (!match) {
        return 1;
    }
    return Math.max(1, leadingSpaces + match[0].length);
}
function formatLocation(diagnostic, root) {
    if (!diagnostic.file) {
        return chalk.bold('<proyecto>');
    }
    const file = path.relative(process.cwd(), path.resolve(root, diagnostic.file));
    const line = diagnostic.line ?? 1;
    const column = diagnostic.column ?? 1;
    return chalk.bold(`${file}:${line}:${column}`);
}
function printSummary(output) {
    if (output?.summary === undefined) {
        return;
    }
    console.log(chalk.dim(`summary: ${JSON.stringify(output.summary)}`));
}
function colorForLevel(level) {
    if (level === 'error') {
        return chalk.red;
    }
    if (level === 'warning') {
        return chalk.yellow;
    }
    return chalk.blue;
}
//# sourceMappingURL=diagnostics.js.map