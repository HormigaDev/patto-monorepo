export declare class CoreBinaryError extends Error {
    readonly code = "patto_cli_core_unavailable";
}
export declare function resolveCoreBinary(): string;
