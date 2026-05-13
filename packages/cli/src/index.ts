#!/usr/bin/env node
import { cac } from "cac";
import chalk from "chalk";
import { registerCoreCommands, writeStructuredError } from "./commands/core.js";
import { registerGenerateCommand } from "./commands/generate.js";
import { registerInitCommand } from "./commands/init.js";

const cli = cac("patto");

cli.version("0.1.0");
cli.help();

registerInitCommand(cli);
registerGenerateCommand(cli);
registerCoreCommands(cli);

try {
    cli.parse(process.argv, { run: false });

    if (cli.matchedCommand === undefined) {
        cli.outputHelp();
    } else {
        await cli.runMatchedCommand();
    }
} catch (error) {
    if (process.argv.includes("--stdin")) {
        writeStructuredError(error);
    } else {
        console.error(
            chalk.red(error instanceof Error ? error.message : String(error)),
        );
    }

    process.exitCode = 1;
}
