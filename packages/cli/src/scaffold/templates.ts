import type { ParsedName } from './names.js';
import { commandClassNameFromPath, commandImportPath, parseDiscordName, toPascalCase } from './names.js';

export type CommandKind = 'command' | 'subcommand' | 'subcommand-group';
export type PluginScopeName = 'specified' | 'folder' | 'deep-folder';

export interface CommandTemplateOptions {
    readonly description?: string;
    readonly category?: string;
}

export interface SubcommandTemplateOptions extends CommandTemplateOptions {
    readonly parent: string;
}

export interface SubcommandGroupTemplateOptions extends CommandTemplateOptions {
    readonly parent: string;
    readonly group: string;
}

export interface PluginRegistrationOptions {
    readonly classBase: string;
    readonly importPath: string;
    readonly scope: PluginScopeName;
    readonly folder?: string;
    readonly commands?: string[];
}

export function commandDefinitionTemplate(
    parsed: ParsedName,
    options: CommandTemplateOptions,
): string {
    return `${decoratorImports('command', options.category)}import { BaseCommand } from '@/core/structures/BaseCommand';

@Command({
    name: '${parsed.name}',
    description: '${quote(options.description ?? `Ejecuta ${parsed.name}`)}',
${categoryLine(options.category)}})
export abstract class ${parsed.classBase}Definition extends BaseCommand {
    // Define aqui tus @Arg cuando el comando necesite parametros.
}
`;
}

export function commandImplementationTemplate(parsed: ParsedName): string {
    return `import { ${parsed.classBase}Definition } from '@/definitions/${[
        ...parsed.dirs,
        `${parsed.fileBase}.definition`,
    ].join('/')}';

export class ${parsed.classBase}Command extends ${parsed.classBase}Definition {
    async run(): Promise<void> {
        await this.send('Comando ${parsed.name} ejecutado.');
    }
}
`;
}

export function commandSingleFileTemplate(
    parsed: ParsedName,
    options: CommandTemplateOptions,
): string {
    return `${decoratorImports('command', options.category)}import { BaseCommand } from '@/core/structures/BaseCommand';

@Command({
    name: '${parsed.name}',
    description: '${quote(options.description ?? `Ejecuta ${parsed.name}`)}',
${categoryLine(options.category)}})
export class ${parsed.classBase}Command extends BaseCommand {
    async run(): Promise<void> {
        await this.send('Comando ${parsed.name} ejecutado.');
    }
}
`;
}

export function subcommandTemplate(
    parsed: ParsedName,
    options: SubcommandTemplateOptions,
): string {
    const parent = parseDiscordName(options.parent, 'comando padre');
    const className = `${toPascalCase(parent)}${parsed.classBase}Command`;

    return `${decoratorImports('subcommand', options.category)}import { BaseCommand } from '@/core/structures/BaseCommand';

@Subcommand({
    parent: '${parent}',
    name: '${parsed.name}',
    description: '${quote(options.description ?? `Ejecuta ${parent} ${parsed.name}`)}',
${categoryLine(options.category)}})
export class ${className} extends BaseCommand {
    async run(): Promise<void> {
        await this.send('Subcomando ${parent} ${parsed.name} ejecutado.');
    }
}
`;
}

export function subcommandGroupTemplate(
    parsed: ParsedName,
    options: SubcommandGroupTemplateOptions,
): string {
    const parent = parseDiscordName(options.parent, 'comando padre');
    const group = parseDiscordName(options.group, 'grupo de subcomandos');
    const className = `${toPascalCase(parent)}${toPascalCase(group)}${parsed.classBase}Command`;

    return `${decoratorImports('subcommand-group', options.category)}import { BaseCommand } from '@/core/structures/BaseCommand';

@SubcommandGroup({
    parent: '${parent}',
    name: '${group}',
    subcommand: '${parsed.name}',
    description: '${quote(options.description ?? `Ejecuta ${parent} ${group} ${parsed.name}`)}',
${categoryLine(options.category)}})
export class ${className} extends BaseCommand {
    async run(): Promise<void> {
        await this.send('Subcomando ${parent} ${group} ${parsed.name} ejecutado.');
    }
}
`;
}

export function standaloneDefinitionTemplate(
    parsed: ParsedName,
    kind: CommandKind,
    options: CommandTemplateOptions & Partial<SubcommandTemplateOptions & SubcommandGroupTemplateOptions>,
): string {
    if (kind === 'subcommand') {
        const parent = parseDiscordName(requiredOption(options.parent, '--parent'), 'comando padre');

        return subcommandTemplate(parsed, {
            ...options,
            parent,
        }).replace(
            `export class ${toPascalCase(parent)}${parsed.classBase}Command extends BaseCommand`,
            `export abstract class ${toPascalCase(parent)}${parsed.classBase}Definition extends BaseCommand`,
        );
    }

    if (kind === 'subcommand-group') {
        const parent = parseDiscordName(requiredOption(options.parent, '--parent'), 'comando padre');
        const group = parseDiscordName(requiredOption(options.group, '--group'), 'grupo de subcomandos');

        return subcommandGroupTemplate(parsed, {
            ...options,
            parent,
            group,
        }).replace(
            `export class ${toPascalCase(parent)}${toPascalCase(group)}${parsed.classBase}Command extends BaseCommand`,
            `export abstract class ${toPascalCase(parent)}${toPascalCase(group)}${parsed.classBase}Definition extends BaseCommand`,
        );
    }

    return commandDefinitionTemplate(parsed, options);
}

export function pluginTemplate(classBase: string): string {
    return `import { BaseCommand } from '@/core/structures/BaseCommand';
import { BasePlugin } from '@/core/structures/BasePlugin';

export class ${classBase}Plugin extends BasePlugin {
    async onBeforeRegisterCommand(
        commandClass: new (...args: any[]) => BaseCommand,
        commandJson: Record<string, unknown>,
    ): Promise<Record<string, unknown> | false | null> {
        return commandJson;
    }

    async onAfterRegisterCommand(
        commandClass: new (...args: any[]) => BaseCommand,
        registeredCommandJson: Record<string, unknown>,
    ): Promise<void> {
        void commandClass;
        void registeredCommandJson;
    }

    async onBeforeExecute(command: BaseCommand): Promise<boolean> {
        void command;
        return true;
    }

    async onAfterExecute(command: BaseCommand): Promise<void> {
        void command;
    }
}
`;
}

export function pluginImportStatement(classBase: string, importPath: string): string {
    return `import { ${classBase}Plugin } from '${importPath}';`;
}

export function pluginCommandImportStatements(commands: string[] | undefined): string[] {
    return (commands ?? []).map((commandPath) => {
        const className = commandClassNameFromPath(commandPath);
        return `import { ${className} } from '${commandImportPath(commandPath)}';`;
    });
}

export function pluginRegistrationTemplate(options: PluginRegistrationOptions): string {
    const scope = pluginScopeEnum(options.scope);
    const commandsLine =
        options.scope === 'specified'
            ? `    commands: [${(options.commands ?? []).map(commandClassNameFromPath).join(', ')}],\n`
            : '';

    return `PluginRegistry.register({
    plugin: new ${options.classBase}Plugin(),
    scope: PluginScope.${scope},
    folderPath: '${quote(options.folder ?? '')}',
${commandsLine}});`;
}

export function pluginScopeEnum(scope: PluginScopeName): 'Specified' | 'Folder' | 'DeepFolder' {
    switch (scope) {
        case 'specified':
            return 'Specified';
        case 'folder':
            return 'Folder';
        case 'deep-folder':
            return 'DeepFolder';
    }
}

export function normalizePluginScope(scope: string | undefined, folder?: string): PluginScopeName {
    if (scope === undefined) {
        return folder ? 'folder' : 'deep-folder';
    }

    if (scope === 'specified' || scope === 'folder' || scope === 'deep-folder') {
        return scope;
    }

    throw new Error('El scope del plugin debe ser specified, folder o deep-folder.');
}

export function normalizeCategory(category: string | undefined): string {
    const value = category?.trim().toLowerCase() || 'other';
    const map: Record<string, string> = {
        economy: 'Economy',
        info: 'Info',
        moderation: 'Moderation',
        other: 'Other',
        settings: 'Settings',
        utils: 'Utils',
    };

    const normalized = map[value];

    if (normalized === undefined) {
        throw new Error('La categoria debe ser info, utils, moderation, settings, economy u other.');
    }

    return normalized;
}

function decoratorImports(kind: CommandKind, category: string | undefined): string {
    const categoryImport = category === undefined ? '' : "import { Category } from '@/utils/CommandCategories';\n";

    if (kind === 'subcommand') {
        return `import { Subcommand } from '@/core/decorators/subcommand.decorator';\n${categoryImport}`;
    }

    if (kind === 'subcommand-group') {
        return `import { SubcommandGroup } from '@/core/decorators/subcommand-group.decorator';\n${categoryImport}`;
    }

    return `import { Command } from '@/core/decorators/command.decorator';\n${categoryImport}`;
}

function categoryLine(category: string | undefined): string {
    return category === undefined ? '' : `    category: Category.${normalizeCategory(category)},\n`;
}

function requiredOption(value: string | undefined, option: string): string {
    if (!value) {
        throw new Error(`Falta ${option}.`);
    }

    return value;
}

function quote(value: string): string {
    return value.replace(/\\/g, '\\\\').replace(/'/g, "\\'");
}
