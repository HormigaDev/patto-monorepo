import type { CoreCommand, CoreRequest } from "./types.js";
export declare function readCoreRequest(defaultCommand?: CoreCommand): Promise<Required<CoreRequest>>;
