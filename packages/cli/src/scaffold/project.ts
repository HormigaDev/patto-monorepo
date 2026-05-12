import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';

export interface ProjectContext {
    readonly root: string;
}

export interface WriteResult {
    readonly path: string;
    readonly created: boolean;
}

export function resolveProject(root?: string): ProjectContext {
    const projectRoot = path.resolve(root ?? process.cwd());
    const requiredFiles = [
        path.join(projectRoot, 'src', 'core', 'structures', 'BaseCommand.ts'),
        path.join(projectRoot, 'src', 'core', 'decorators', 'command.decorator.ts'),
    ];

    const missingFile = requiredFiles.find((file) => !existsSync(file));

    if (missingFile !== undefined) {
        throw new Error(
            `No parece ser un proyecto Patto válido. No encontré ${path.relative(
                projectRoot,
                missingFile,
            )}. Usa --root para indicar la raíz del bot.`,
        );
    }

    return { root: projectRoot };
}

export function projectPath(project: ProjectContext, ...segments: string[]): string {
    const target = path.resolve(project.root, ...segments);
    const relative = path.relative(project.root, target);

    if (relative.startsWith('..') || path.isAbsolute(relative)) {
        throw new Error(`La ruta ${target} intenta salir de la raíz del proyecto.`);
    }

    return target;
}

export function writeFileOnce(filePath: string, content: string, force: boolean): WriteResult {
    if (existsSync(filePath) && !force) {
        throw new Error(`El archivo ya existe: ${filePath}. Usa --force para reemplazarlo.`);
    }

    mkdirSync(path.dirname(filePath), { recursive: true });
    writeFileSync(filePath, content, 'utf8');

    return { path: filePath, created: true };
}

export function readTextFile(filePath: string): string {
    return readFileSync(filePath, 'utf8');
}

export function writeTextFile(filePath: string, content: string): void {
    mkdirSync(path.dirname(filePath), { recursive: true });
    writeFileSync(filePath, content, 'utf8');
}

export function fileExists(filePath: string): boolean {
    return existsSync(filePath);
}
