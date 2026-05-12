export type PattoCoreCommand = 'lint' | 'check';
export type PattoDiagnosticLevel = 'error' | 'warning' | 'info';

export interface PattoDiagnostic {
    readonly level: PattoDiagnosticLevel;
    readonly code: string;
    readonly message: string;
    readonly file?: string | null;
    readonly line?: number | null;
    readonly column?: number | null;
    readonly hint?: string | null;
}

export interface PattoCoreEnvelope {
    readonly ok: boolean;
    readonly command: PattoCoreCommand;
    readonly exitCode: number;
    readonly stderr: string;
    readonly diagnostics: PattoDiagnostic[];
    readonly output?: unknown;
}
