import type { CAC, Command } from "cac";
import chalk from "chalk";
import path from "node:path";
import {
    commandDefinitionTemplate,
    commandImplementationTemplate,
    commandSingleFileTemplate,
    normalizeCategory,
    normalizePluginScope,
    pluginCommandImportStatements,
    pluginImportStatement,
    pluginRegistrationTemplate,
    pluginTemplate,
    standaloneDefinitionTemplate,
    subcommandGroupTemplate,
    subcommandTemplate,
    type CommandKind,
    type PluginScopeName,
} from "../scaffold/templates.js";
import {
    commandImportPath,
    parseDiscordName,
    parsePathSegments,
    parseScaffoldName,
} from "../scaffold/names.js";
import {
    fileExists,
    projectPath,
    readTextFile,
    resolveProject,
    writeFileOnce,
    writeTextFile,
    type ProjectContext,
    type WriteResult,
} from "../scaffold/project.js";

interface CommonOptions {
    readonly root?: string;
    readonly force?: boolean;
    readonly description?: string;
    readonly category?: string;
}

interface CommandOptions extends CommonOptions {
    readonly singleFile?: boolean;
    readonly unified?: boolean;
}

interface SubcommandOptions extends CommonOptions {
    readonly parent?: string;
    readonly group?: string;
}

interface DefinitionOptions extends CommonOptions {
    readonly kind?: CommandKind;
    readonly parent?: string;
    readonly group?: string;
}

interface PluginOptions {
    readonly root?: string;
    readonly force?: boolean;
    readonly scope?: string;
    readonly folder?: string;
    readonly commands?: string;
    readonly register?: boolean;
}

export function registerGenerateCommand(cli: CAC): void {
    registerGenerateAlias(
        cli.command("generate <type> [name]", "Genera scaffolds Patto"),
    );
    registerGenerateAlias(cli.command("g <type> [name]", "Alias de generate"));
    registerGenerateAlias(
        cli.command("scaffold <type> [name]", "Alias de generate"),
    );
}

function registerGenerateAlias(command: Command): void {
    withGenerateOptions(command).action(runAction(dispatchGenerate));
}

function dispatchGenerate(
    type: string,
    name: string | undefined,
    options: CommandOptions &
        SubcommandOptions &
        DefinitionOptions &
        PluginOptions,
): void {
    const scaffoldName = required(name, "nombre");

    switch (type) {
        case "command":
        case "cmd":
            generateCommand(scaffoldName, options);
            return;
        case "subcommand":
        case "sub":
            generateSubcommand(scaffoldName, options);
            return;
        case "subcommand-group":
        case "group":
            generateSubcommandGroup(scaffoldName, options);
            return;
        case "definition":
        case "def":
            generateDefinition(scaffoldName, options);
            return;
        case "plugin":
            generatePlugin(scaffoldName, options);
            return;
        default:
            throw new Error(
                "Tipo de scaffold invalido. Usa command, subcommand, subcommand-group, definition o plugin.",
            );
    }
}

function generateCommand(name: string, options: CommandOptions): void {
    const project = resolveProject(options.root);
    const parsed = parseScaffoldName(name, "comando");
    const category = normalizeCategory(options.category);
    const commandFile = projectPath(
        project,
        "src",
        "commands",
        ...parsed.dirs,
        `${parsed.fileBase}.command.ts`,
    );

    if (options.singleFile || options.unified) {
        const result = writeFileOnce(
            commandFile,
            commandSingleFileTemplate(parsed, { ...options, category }),
            Boolean(options.force),
        );
        printCreated([result], project);
        return;
    }

    const definitionFile = projectPath(
        project,
        "src",
        "definitions",
        ...parsed.dirs,
        `${parsed.fileBase}.definition.ts`,
    );
    const results = [
        writeFileOnce(
            definitionFile,
            commandDefinitionTemplate(parsed, { ...options, category }),
            Boolean(options.force),
        ),
        writeFileOnce(
            commandFile,
            commandImplementationTemplate(parsed),
            Boolean(options.force),
        ),
    ];

    printCreated(results, project);
}

function generateSubcommand(name: string, options: SubcommandOptions): void {
    const project = resolveProject(options.root);
    const parsed = parseScaffoldName(name, "subcomando");
    const parent = parseDiscordName(
        required(options.parent, "--parent"),
        "comando padre",
    );
    const dirs = parsed.dirs.length > 0 ? parsed.dirs : [parent];
    const commandFile = projectPath(
        project,
        "src",
        "commands",
        ...dirs,
        `${parsed.fileBase}.command.ts`,
    );
    const result = writeFileOnce(
        commandFile,
        subcommandTemplate(parsed, {
            ...options,
            parent,
            category: normalizeCategory(options.category),
        }),
        Boolean(options.force),
    );

    printCreated([result], project);
}

function generateSubcommandGroup(
    name: string,
    options: SubcommandOptions,
): void {
    const project = resolveProject(options.root);
    const parsed = parseScaffoldName(name, "subcomando");
    const parent = parseDiscordName(
        required(options.parent, "--parent"),
        "comando padre",
    );
    const group = parseDiscordName(required(options.group, "--group"), "grupo");
    const dirs = parsed.dirs.length > 0 ? parsed.dirs : [parent, group];
    const commandFile = projectPath(
        project,
        "src",
        "commands",
        ...dirs,
        `${parsed.fileBase}.command.ts`,
    );
    const result = writeFileOnce(
        commandFile,
        subcommandGroupTemplate(parsed, {
            ...options,
            parent,
            group,
            category: normalizeCategory(options.category),
        }),
        Boolean(options.force),
    );

    printCreated([result], project);
}

function generateDefinition(name: string, options: DefinitionOptions): void {
    const project = resolveProject(options.root);
    const parsed = parseScaffoldName(name, "definition");
    const kind = normalizeDefinitionKind(options.kind);
    const definitionFile = projectPath(
        project,
        "src",
        "definitions",
        ...parsed.dirs,
        `${parsed.fileBase}.definition.ts`,
    );
    const result = writeFileOnce(
        definitionFile,
        standaloneDefinitionTemplate(parsed, kind, {
            ...options,
            category: normalizeCategory(options.category),
        }),
        Boolean(options.force),
    );

    printCreated([result], project);
}

function generatePlugin(name: string, options: PluginOptions): void {
    const project = resolveProject(options.root);
    const parsed = parseScaffoldName(name, "plugin");
    const scope = normalizePluginScope(options.scope, options.folder);
    const folder = normalizePluginFolder(scope, options.folder);
    const commands = parseCommandList(options.commands);

    if (scope === "specified" && commands.length === 0) {
        throw new Error(
            "PluginScope.Specified necesita --commands admin/ban,info/ping.",
        );
    }

    const pluginFile = projectPath(
        project,
        "src",
        "plugins",
        ...parsed.dirs,
        `${parsed.fileBase}.plugin.ts`,
    );
    const result = writeFileOnce(
        pluginFile,
        pluginTemplate(parsed.classBase),
        Boolean(options.force),
    );

    if (options.register !== false) {
        registerPlugin(project, {
            classBase: parsed.classBase,
            commands,
            folder,
            importPath: `@/plugins/${[...parsed.dirs, `${parsed.fileBase}.plugin`].join("/")}`,
            scope,
        });
    }

    printCreated([result], project);
}

function registerPlugin(
    project: ProjectContext,
    options: {
        readonly classBase: string;
        readonly commands: string[];
        readonly folder: string;
        readonly importPath: string;
        readonly scope: PluginScopeName;
    },
): void {
    const configFile = projectPath(
        project,
        "src",
        "config",
        "plugins.config.ts",
    );
    let content = fileExists(configFile) ? readTextFile(configFile) : "";
    const imports = [
        "import { PluginRegistry, PluginScope } from './plugin.registry';",
        pluginImportStatement(options.classBase, options.importPath),
        ...pluginCommandImportStatements(options.commands),
    ];
    const registration = pluginRegistrationTemplate(options);

    content = addImports(content, imports);

    if (!content.includes(registration)) {
        content = `${content.trimEnd()}\n\n${registration}\n`;
    }

    writeTextFile(configFile, content);
    console.log(
        chalk.green(
            `Plugin registrado en ${path.relative(project.root, configFile)}`,
        ),
    );
}

function addImports(content: string, imports: string[]): string {
    let next = content.trimStart();

    for (const importStatement of imports) {
        if (next.includes(importStatement)) {
            continue;
        }

        const lastImportEnd = findLastImportEnd(next);
        next =
            lastImportEnd === 0
                ? `${importStatement}\n${next}`
                : `${next.slice(0, lastImportEnd)}${importStatement}\n${next.slice(lastImportEnd)}`;
    }

    return next;
}

function findLastImportEnd(content: string): number {
    const importRegex = /^import .+;\n/gm;
    let lastEnd = 0;
    let match: RegExpExecArray | null;

    while ((match = importRegex.exec(content)) !== null) {
        lastEnd = match.index + match[0].length;
    }

    return lastEnd;
}

function normalizePluginFolder(
    scope: PluginScopeName,
    folder: string | undefined,
): string {
    if (scope === "specified") {
        return "";
    }

    if (scope === "folder" && !folder) {
        throw new Error("PluginScope.Folder necesita --folder.");
    }

    return folder ? parsePathSegments(folder, "folder").join("/") : "";
}

function parseCommandList(input: string | undefined): string[] {
    if (!input) {
        return [];
    }

    return input
        .split(",")
        .map((item) => item.trim())
        .filter(Boolean)
        .map((item) => {
            parseScaffoldName(item, "comando");
            return item;
        });
}

function normalizeDefinitionKind(kind: CommandKind | undefined): CommandKind {
    if (kind === undefined) {
        return "command";
    }

    if (
        kind === "command" ||
        kind === "subcommand" ||
        kind === "subcommand-group"
    ) {
        return kind;
    }

    throw new Error("--kind debe ser command, subcommand o subcommand-group.");
}

function withGenerateOptions(command: Command): Command {
    return command
        .option("--root <path>", "Raiz del proyecto Patto")
        .option("--force", "Reemplaza archivos existentes")
        .option("-d, --description <text>", "Descripcion del comando")
        .option(
            "-c, --category <name>",
            "info, utils, moderation, settings, economy u other",
        )
        .option("--single-file", "Crea solo el archivo .command.ts")
        .option("-u, --unified", "Alias de --single-file")
        .option("--kind <kind>", "command, subcommand o subcommand-group")
        .option("-p, --parent <name>", "Nombre del comando padre")
        .option("-g, --group <name>", "Nombre del grupo")
        .option("--scope <scope>", "specified, folder o deep-folder")
        .option(
            "--folder <path>",
            "Carpeta de comandos para folder/deep-folder",
        )
        .option(
            "--commands <paths>",
            "Lista separada por coma para scope specified",
        )
        .option("--no-register", "No modifica src/config/plugins.config.ts");
}

function runAction<T extends unknown[]>(
    handler: (...args: T) => void | Promise<void>,
): (...args: T) => Promise<void> {
    return async (...args: T) => {
        try {
            await handler(...args);
        } catch (error) {
            console.error(chalk.red(formatError(error)));
            process.exitCode = 1;
        }
    };
}

function printCreated(results: WriteResult[], project: ProjectContext): void {
    for (const result of results) {
        console.log(
            chalk.green(`Creado ${path.relative(project.root, result.path)}`),
        );
    }
}

function required(value: string | undefined, option: string): string {
    if (!value) {
        throw new Error(`Falta ${option}.`);
    }

    return value;
}

function formatError(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
}
