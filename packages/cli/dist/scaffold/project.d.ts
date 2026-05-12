export interface ProjectContext {
    readonly root: string;
}
export interface WriteResult {
    readonly path: string;
    readonly created: boolean;
}
export declare function resolveProject(root?: string): ProjectContext;
export declare function projectPath(project: ProjectContext, ...segments: string[]): string;
export declare function writeFileOnce(filePath: string, content: string, force: boolean): WriteResult;
export declare function readTextFile(filePath: string): string;
export declare function writeTextFile(filePath: string, content: string): void;
export declare function fileExists(filePath: string): boolean;
