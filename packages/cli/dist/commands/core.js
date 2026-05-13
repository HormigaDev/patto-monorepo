import path from "node:path";
import { runCore } from "../core/run.js";
import { readCoreRequest } from "../core/stdin.js";
import { diagnosticsFromOutput, printHumanOutput, } from "../output/diagnostics.js";
export function registerCoreCommands(cli) {
    registerCoreCommand(cli.command("scan", "Indexa el proyecto Patto"), "scan");
    registerCoreCommand(cli.command("lint", "Ejecuta reglas estaticas de Patto"), "lint");
    registerCoreCommand(cli.command("doctor", "Revisa salud del entorno Patto"), "doctor");
    registerCoreCommand(cli.command("check", "Ejecuta scan + lint + doctor"), "check");
    cli.command("core", "API JSON por stdin para extensiones")
        .option("--stdin", "Lee un request JSON desde stdin")
        .action(async (options) => {
        if (!options.stdin) {
            throw new Error("Usa patto core --stdin.");
        }
        await runFromStdin();
    });
}
function registerCoreCommand(command, coreCommand) {
    command
        .option("--root <path>", "Raiz del proyecto Patto")
        .option("--lang <lang>", "Idioma del core: auto o es")
        .option("--json", "Imprime la salida JSON cruda del core")
        .option("--stdin", "Lee root/lang desde un request JSON en stdin")
        .action(async (options) => {
        if (options.stdin) {
            await runFromStdin(coreCommand);
            return;
        }
        await runDirect(coreCommand, options);
    });
}
async function runDirect(command, options) {
    const root = path.resolve(options.root ?? process.cwd());
    const result = await runCore({
        command,
        root,
        lang: options.lang,
    });
    if (options.json) {
        process.stdout.write(result.stdout);
        if (!result.stdout.endsWith("\n")) {
            process.stdout.write("\n");
        }
    }
    else {
        if (result.stderr.trim().length > 0) {
            process.stderr.write(result.stderr);
        }
        printHumanOutput(result.output, { root, command });
    }
    process.exitCode = result.exitCode;
}
async function runFromStdin(defaultCommand) {
    const request = await readCoreRequest(defaultCommand);
    const result = await runCore(request);
    const envelope = {
        ok: result.exitCode === 0,
        command: result.command,
        exitCode: result.exitCode,
        stderr: result.stderr,
        output: result.output,
        diagnostics: diagnosticsFromOutput(result.output),
    };
    process.stdout.write(`${JSON.stringify(envelope, null, 2)}\n`);
    process.exitCode = result.exitCode;
}
export function writeStructuredError(error) {
    const message = error instanceof Error ? error.message : String(error);
    const code = error instanceof Error &&
        "code" in error &&
        typeof error.code === "string"
        ? error.code
        : "patto_cli_error";
    const payload = {
        ok: false,
        error: {
            message,
            code,
        },
    };
    process.stderr.write(`${JSON.stringify(payload, null, 2)}\n`);
}
//# sourceMappingURL=core.js.map