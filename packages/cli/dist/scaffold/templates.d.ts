import type { ParsedName } from "./names.js";
export type CommandKind = "command" | "subcommand" | "subcommand-group";
export type PluginScopeName = "specified" | "folder" | "deep-folder";
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
export declare function commandDefinitionTemplate(parsed: ParsedName, options: CommandTemplateOptions): string;
export declare function commandImplementationTemplate(parsed: ParsedName): string;
export declare function commandSingleFileTemplate(parsed: ParsedName, options: CommandTemplateOptions): string;
export declare function subcommandTemplate(parsed: ParsedName, options: SubcommandTemplateOptions): string;
export declare function subcommandGroupTemplate(parsed: ParsedName, options: SubcommandGroupTemplateOptions): string;
export declare function standaloneDefinitionTemplate(parsed: ParsedName, kind: CommandKind, options: CommandTemplateOptions & Partial<SubcommandTemplateOptions & SubcommandGroupTemplateOptions>): string;
export declare function pluginTemplate(classBase: string): string;
export declare function pluginImportStatement(classBase: string, importPath: string): string;
export declare function pluginCommandImportStatements(commands: string[] | undefined): string[];
export declare function pluginRegistrationTemplate(options: PluginRegistrationOptions): string;
export declare function pluginScopeEnum(scope: PluginScopeName): "Specified" | "Folder" | "DeepFolder";
export declare function normalizePluginScope(scope: string | undefined, folder?: string): PluginScopeName;
export declare function normalizeCategory(category: string | undefined): string;
