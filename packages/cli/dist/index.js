#!/usr/bin/env node
import { cac } from 'cac';
import chalk from 'chalk';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { registerCoreCommands, writeStructuredError } from './commands/core.js';
import { registerGenerateCommand } from './commands/generate.js';
import { registerInitCommand } from './commands/init.js';
const pkgJson = JSON.parse(readFileSync(path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'package.json'), 'utf8'));
const cli = cac('patto');
cli.version(pkgJson.version);
cli.help();
registerInitCommand(cli);
registerGenerateCommand(cli);
registerCoreCommands(cli);
try {
    // cac con `run: false` llama outputHelp()/outputVersion() cuando detecta los flags
    // y luego borra matchedCommand via unsetMatchedCommand(). Capturamos los flags ANTES
    // del parse para no volver a imprimir en nuestra logica de despacho.
    const handledByCac = process.argv.includes('--help') ||
        process.argv.includes('-h') ||
        process.argv.includes('--version') ||
        process.argv.includes('-v');
    cli.parse(process.argv, { run: false });
    if (handledByCac) {
        // cac ya imprimio la salida correcta durante parse(); no hacer nada mas.
    }
    else if (cli.matchedCommand === undefined) {
        cli.outputHelp();
    }
    else {
        await cli.runMatchedCommand();
    }
}
catch (error) {
    if (process.argv.includes('--stdin')) {
        writeStructuredError(error);
    }
    else {
        console.error(chalk.red(error instanceof Error ? error.message : String(error)));
    }
    process.exitCode = 1;
}
//# sourceMappingURL=index.js.map