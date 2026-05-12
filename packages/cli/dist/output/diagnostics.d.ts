import type { CoreDiagnostic, CoreOutput } from '../core/types.js';
interface RenderOptions {
    readonly root: string;
    readonly command: string;
}
export declare function printHumanOutput(output: CoreOutput | null, options: RenderOptions): void;
export declare function diagnosticsFromOutput(output: CoreOutput | null): CoreDiagnostic[];
export {};
