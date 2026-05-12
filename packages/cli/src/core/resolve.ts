import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { existsSync } from 'node:fs';

const require = createRequire(import.meta.url);
const currentFile = fileURLToPath(import.meta.url);
const currentDir = path.dirname(currentFile);

interface PlatformPackage {
    readonly name: string;
    readonly workspaceDir: string;
    readonly binaryName: string;
}

export class CoreBinaryError extends Error {
    readonly code = 'patto_cli_core_unavailable';
}

export function resolveCoreBinary(): string {
    const platformPackage = getPlatformPackage();

    if (platformPackage === undefined) {
        throw new CoreBinaryError(
            `Patto CLI aun no tiene binario compatible para ${process.platform}-${process.arch}. ` +
                'Plataformas soportadas: linux-x64, linux-arm64 y win32-x64.',
        );
    }

    const packageBinary = resolvePackageBinary(platformPackage);

    if (packageBinary !== undefined) {
        return packageBinary;
    }

    const workspaceBinary = path.resolve(
        currentDir,
        '..',
        '..',
        '..',
        platformPackage.workspaceDir,
        'bin',
        platformPackage.binaryName,
    );

    if (existsSync(workspaceBinary)) {
        return workspaceBinary;
    }

    throw new CoreBinaryError(
        `No encontre el binario nativo ${platformPackage.name}. ` +
            'Ejecuta el build del core o reinstala @patto/cli para descargar la optionalDependency correcta.',
    );
}

function resolvePackageBinary(platformPackage: PlatformPackage): string | undefined {
    try {
        const packageJsonPath = require.resolve(`${platformPackage.name}/package.json`);
        const binaryPath = path.join(
            path.dirname(packageJsonPath),
            'bin',
            platformPackage.binaryName,
        );

        return existsSync(binaryPath) ? binaryPath : undefined;
    } catch {
        return undefined;
    }
}

function getPlatformPackage(): PlatformPackage | undefined {
    if (process.platform === 'linux' && process.arch === 'x64') {
        return {
            name: '@patto/cli-core-linux-x64',
            workspaceDir: 'cli-core-linux-x64',
            binaryName: 'patto-core',
        };
    }

    if (process.platform === 'linux' && process.arch === 'arm64') {
        return {
            name: '@patto/cli-core-linux-arm64',
            workspaceDir: 'cli-core-linux-arm64',
            binaryName: 'patto-core',
        };
    }

    if (process.platform === 'win32' && process.arch === 'x64') {
        return {
            name: '@patto/cli-core-win32-x64',
            workspaceDir: 'cli-core-win32-x64',
            binaryName: 'patto-core.exe',
        };
    }

    return undefined;
}
