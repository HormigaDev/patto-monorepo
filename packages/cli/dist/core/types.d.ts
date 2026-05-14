export type CoreCommand = "scan" | "lint" | "doctor" | "check" | "format-i18n";
export type DiagnosticLevel = "error" | "warning" | "info";
export interface CoreDiagnostic {
    readonly level: DiagnosticLevel;
    readonly code: string;
    readonly message: string;
    readonly file?: string | null;
    readonly line?: number | null;
    readonly column?: number | null;
    readonly hint?: string | null;
}
export interface CoreOutput {
    readonly status?: string;
    readonly command?: string;
    readonly diagnostics?: CoreDiagnostic[];
    readonly summary?: unknown;
    readonly stats?: unknown;
    readonly [key: string]: unknown;
}
export interface CoreRequest {
    readonly command?: CoreCommand;
    readonly root?: string;
    readonly lang?: string;
}
export interface CoreRunOptions {
    readonly command: CoreCommand;
    readonly root?: string;
    readonly lang?: string;
}
export interface CoreRunResult {
    readonly command: CoreCommand;
    readonly exitCode: number;
    readonly stdout: string;
    readonly stderr: string;
    readonly output: CoreOutput | null;
}
export interface CoreEnvelope {
    readonly ok: boolean;
    readonly command: CoreCommand;
    readonly exitCode: number;
    readonly stderr: string;
    readonly output: CoreOutput | null;
    readonly diagnostics: CoreDiagnostic[];
}
