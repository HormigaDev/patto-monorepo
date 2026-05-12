export interface ParsedName {
    readonly input: string;
    readonly name: string;
    readonly fileBase: string;
    readonly classBase: string;
    readonly dirs: string[];
    readonly parts: string[];
}
export declare function parseScaffoldName(input: string, label: string): ParsedName;
export declare function parseDiscordName(input: string, label: string): string;
export declare function parsePathSegments(input: string, label: string): string[];
export declare function toPascalCase(value: string): string;
export declare function commandClassNameFromPath(input: string): string;
export declare function commandImportPath(input: string): string;
