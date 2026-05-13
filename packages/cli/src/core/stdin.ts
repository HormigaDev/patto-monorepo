import { stdin } from "node:process";
import type { CoreCommand, CoreRequest } from "./types.js";

export async function readCoreRequest(
    defaultCommand?: CoreCommand,
): Promise<Required<CoreRequest>> {
    const raw = await readStdin();
    const parsed =
        raw.trim().length === 0 ? {} : (JSON.parse(raw) as CoreRequest);
    const command = parsed.command ?? defaultCommand;

    if (command === undefined) {
        throw new Error('El JSON de stdin debe incluir "command".');
    }

    if (!isCoreCommand(command)) {
        throw new Error("El comando debe ser scan, lint, doctor o check.");
    }

    return {
        command,
        root: parsed.root ?? process.cwd(),
        lang: parsed.lang ?? "auto",
    };
}

function readStdin(): Promise<string> {
    return new Promise((resolve, reject) => {
        const chunks: Buffer[] = [];

        stdin.on("data", (chunk: Buffer) => chunks.push(chunk));
        stdin.on("error", reject);
        stdin.on("end", () => resolve(Buffer.concat(chunks).toString("utf8")));
        stdin.resume();
    });
}

function isCoreCommand(value: string): value is CoreCommand {
    return (
        value === "scan" ||
        value === "lint" ||
        value === "doctor" ||
        value === "check"
    );
}
