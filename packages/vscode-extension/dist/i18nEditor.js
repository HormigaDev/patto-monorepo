"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.registerI18nTranslationEditor = registerI18nTranslationEditor;
const vscode = __importStar(require("vscode"));
const node_fs_1 = require("node:fs");
const promises_1 = require("node:fs/promises");
const node_path_1 = __importDefault(require("node:path"));
const cli_1 = require("./cli");
const workspace_1 = require("./workspace");
function registerI18nTranslationEditor(output) {
    const selector = [
        { scheme: "file", language: "typescript" },
        { scheme: "file", language: "typescriptreact" },
        { scheme: "file", language: "javascript" },
        { scheme: "file", language: "javascriptreact" },
    ];
    const keyDecoration = vscode.window.createTextEditorDecorationType({
        textDecoration: "underline dotted",
    });
    const refreshVisibleEditors = () => {
        for (const editor of vscode.window.visibleTextEditors) {
            refreshI18nKeyDecorations(editor, keyDecoration);
        }
    };
    refreshVisibleEditors();
    return [
        keyDecoration,
        vscode.languages.registerHoverProvider(selector, {
            async provideHover(document, position) {
                const match = findStaticI18nKeyAtPosition(document, position);
                if (!match) {
                    return undefined;
                }
                const folder = vscode.workspace.getWorkspaceFolder(document.uri);
                if (!folder || !(0, workspace_1.isPattoWorkspaceFolder)(folder)) {
                    return undefined;
                }
                const localePath = spanishLocalePath(folder.uri.fsPath);
                if (!(0, node_fs_1.existsSync)(localePath)) {
                    return undefined;
                }
                const currentValue = await readCurrentSpanishValue(localePath, match.key);
                const commandUri = i18nEditCommandUri({
                    key: match.key,
                    documentUri: document.uri.toString(),
                    line: position.line,
                    character: position.character,
                });
                const label = currentValue === undefined
                    ? "$(add) AGREGAR TRADUCCION I18N"
                    : "$(edit) EDITAR TRADUCCION I18N";
                const color = currentValue === undefined ? "#3fb950" : "#58a6ff";
                const markdown = new vscode.MarkdownString(`[<span style="color:${color};">${label}</span>](${commandUri})`, true);
                markdown.isTrusted = {
                    enabledCommands: ["patto.editI18nTranslation"],
                };
                markdown.supportHtml = true;
                markdown.supportThemeIcons = true;
                return new vscode.Hover(markdown, match.range);
            },
        }),
        vscode.commands.registerCommand("patto.editI18nTranslation", async (keyOrArgs, documentUri) => {
            await editI18nTranslation(normalizeEditArgs(keyOrArgs, documentUri), output);
        }),
        vscode.languages.registerDocumentLinkProvider(selector, {
            provideDocumentLinks(document) {
                const folder = vscode.workspace.getWorkspaceFolder(document.uri);
                if (!folder ||
                    !(0, workspace_1.isPattoWorkspaceFolder)(folder) ||
                    !(0, node_fs_1.existsSync)(spanishLocalePath(folder.uri.fsPath))) {
                    return [];
                }
                return findStaticI18nKeyMatches(document).map((match) => {
                    const link = new vscode.DocumentLink(match.range, vscode.Uri.parse(i18nEditCommandUri({
                        key: match.key,
                        documentUri: document.uri.toString(),
                        line: match.range.start.line,
                        character: match.range.start.character,
                    })));
                    link.tooltip = "Agregar o editar traduccion i18n";
                    return link;
                });
            },
        }),
        vscode.window.onDidChangeVisibleTextEditors(refreshVisibleEditors),
        vscode.workspace.onDidChangeTextDocument((event) => {
            for (const editor of vscode.window.visibleTextEditors) {
                if (editor.document === event.document) {
                    refreshI18nKeyDecorations(editor, keyDecoration);
                }
            }
        }),
    ];
}
function normalizeEditArgs(keyOrArgs, documentUri) {
    if (typeof keyOrArgs === "string") {
        return { key: keyOrArgs, documentUri };
    }
    return keyOrArgs ?? {};
}
function i18nEditCommandUri(args) {
    return `command:patto.editI18nTranslation?${encodeURIComponent(JSON.stringify([args]))}`;
}
function refreshI18nKeyDecorations(editor, decoration) {
    const document = editor.document;
    if (!isSupportedSourceDocument(document)) {
        editor.setDecorations(decoration, []);
        return;
    }
    const folder = vscode.workspace.getWorkspaceFolder(document.uri);
    if (!folder ||
        !(0, workspace_1.isPattoWorkspaceFolder)(folder) ||
        !(0, node_fs_1.existsSync)(spanishLocalePath(folder.uri.fsPath))) {
        editor.setDecorations(decoration, []);
        return;
    }
    editor.setDecorations(decoration, findStaticI18nKeyMatches(document).map((match) => match.range));
}
function isSupportedSourceDocument(document) {
    return (document.uri.scheme === "file" &&
        (document.languageId === "typescript" ||
            document.languageId === "typescriptreact" ||
            document.languageId === "javascript" ||
            document.languageId === "javascriptreact"));
}
function findStaticI18nKeyAtPosition(document, position) {
    const offset = document.offsetAt(position);
    return findStaticI18nKeyMatches(document).find((match) => {
        const start = document.offsetAt(match.range.start);
        const end = document.offsetAt(match.range.end);
        return offset >= Math.max(0, start - 1) && offset <= end + 1;
    });
}
function findStaticI18nKeyMatches(document) {
    const text = document.getText();
    const aliases = findThisTAliases(text);
    return [
        ...findThisTCalls(text),
        ...aliases.flatMap((alias) => findAliasCalls(text, alias)),
    ];
}
function findThisTAliases(text) {
    const aliases = [];
    const aliasPattern = /\b(?:const|let|var)\s+([A-Za-z_$][\w$]*)\s*=\s*this\s*\.\s*t(?:\s*\.\s*bind\s*\(\s*this\s*\))?\s*;?/g;
    let match;
    while ((match = aliasPattern.exec(text)) !== null) {
        if (isCodePosition(text, match.index)) {
            aliases.push({
                name: match[1],
                declarationEnd: aliasPattern.lastIndex,
            });
        }
    }
    return aliases;
}
function findThisTCalls(text) {
    return findStaticCalls(text, /this\s*\.\s*t\s*\(\s*(['"])([^'"`\r\n]+)\1/g);
}
function findAliasCalls(text, alias) {
    return findStaticCalls(text, new RegExp("\\b" +
        escapeRegExp(alias.name) +
        "\\s*\\(\\s*(['\"])([^'\"`\\r\\n]+)\\1", "g"), alias.declarationEnd);
}
function findStaticCalls(text, pattern, minOffset = 0) {
    const matches = [];
    let match;
    while ((match = pattern.exec(text)) !== null) {
        const key = match[2];
        if (match.index < minOffset ||
            !isCodePosition(text, match.index) ||
            !isStaticTranslationKey(key) ||
            !hasStaticFirstArgumentTerminator(text, pattern.lastIndex)) {
            continue;
        }
        const quotedKey = `${match[1]}${key}${match[1]}`;
        const keyStartInCall = match[0].lastIndexOf(quotedKey) + 1;
        if (keyStartInCall <= 0) {
            continue;
        }
        const keyStart = match.index + keyStartInCall;
        const keyEnd = keyStart + key.length;
        matches.push({
            key,
            range: new vscode.Range(positionAt(text, keyStart), positionAt(text, keyEnd)),
        });
    }
    return matches;
}
function isCodePosition(text, offset) {
    let state = "normal";
    for (let index = 0; index < offset; index += 1) {
        const char = text[index];
        const next = text[index + 1];
        if (state === "normal") {
            if (char === "'") {
                state = "single";
            }
            else if (char === '"') {
                state = "double";
            }
            else if (char === "`") {
                state = "template";
            }
            else if (char === "/" && next === "/") {
                state = "line-comment";
                index += 1;
            }
            else if (char === "/" && next === "*") {
                state = "block-comment";
                index += 1;
            }
        }
        else if (state === "single") {
            if (char === "\\") {
                index += 1;
            }
            else if (char === "'") {
                state = "normal";
            }
        }
        else if (state === "double") {
            if (char === "\\") {
                index += 1;
            }
            else if (char === '"') {
                state = "normal";
            }
        }
        else if (state === "template") {
            if (char === "\\") {
                index += 1;
            }
            else if (char === "`") {
                state = "normal";
            }
        }
        else if (state === "line-comment") {
            if (char === "\n") {
                state = "normal";
            }
        }
        else if (state === "block-comment" && char === "*" && next === "/") {
            state = "normal";
            index += 1;
        }
    }
    return state === "normal";
}
function hasStaticFirstArgumentTerminator(text, offset) {
    let index = offset;
    while (index < text.length && /\s/.test(text[index])) {
        index += 1;
    }
    return text[index] === ")" || text[index] === ",";
}
function isStaticTranslationKey(value) {
    return /^[A-Za-z0-9_.:-]+$/.test(value);
}
function positionAt(text, offset) {
    const prefix = text.slice(0, offset);
    const lines = prefix.split(/\r?\n/);
    return new vscode.Position(lines.length - 1, lines[lines.length - 1].length);
}
async function editI18nTranslation(args, output) {
    const resolved = resolveEditTarget(args);
    if (!resolved || !isStaticTranslationKey(resolved.key)) {
        vscode.window.showWarningMessage("Pon el cursor dentro de una key i18n estatica.");
        return;
    }
    const { key, documentUri, restorePosition } = resolved;
    const folder = workspaceFolderFromDocumentUri(documentUri) ??
        vscode.workspace.workspaceFolders?.[0];
    if (!folder || !(0, workspace_1.isPattoWorkspaceFolder)(folder)) {
        vscode.window.showWarningMessage("No se encontro un workspace Patto activo.");
        return;
    }
    const localePath = spanishLocalePath(folder.uri.fsPath);
    if (!(0, node_fs_1.existsSync)(localePath)) {
        vscode.window.showWarningMessage("No existe src/i18n/locale/es.ts en este proyecto.");
        return;
    }
    const currentValue = await readCurrentSpanishValue(localePath, key);
    if (currentValue === null) {
        vscode.window.showWarningMessage("Esta traduccion existe, pero no es un string editable desde el hover.");
        return;
    }
    const value = await vscode.window.showInputBox({
        title: currentValue === undefined ? `Agregar ${key}` : `Editar ${key}`,
        prompt: "Valor en es.ts. Deja vacio para cancelar.",
        value: currentValue ?? "",
        ignoreFocusOut: true,
    });
    await hideI18nHover();
    await restoreEditorPosition(documentUri, restorePosition);
    if (value === undefined || value === "") {
        return;
    }
    if (value === currentValue) {
        return;
    }
    await upsertSpanishTranslation(localePath, key, value);
    const cliCommand = await (0, cli_1.resolvePattoCli)();
    if (!cliCommand) {
        vscode.window.showWarningMessage("Traduccion guardada, pero no se encontro patto CLI para ejecutar format-i18n.");
        return;
    }
    const result = await (0, cli_1.runPattoFormatI18n)(cliCommand, folder.uri.fsPath);
    if (result.stderr.trim().length > 0) {
        output.appendLine(result.stderr);
    }
    if (result.exitCode !== 0) {
        output.appendLine(result.stdout);
        output.show(true);
        vscode.window.showWarningMessage("Traduccion guardada, pero patto format-i18n reporto diagnosticos.");
        return;
    }
    vscode.window.showInformationMessage(`Traduccion i18n guardada: ${key}`);
}
function resolveEditTarget(args) {
    const editor = args.documentUri
        ? vscode.window.visibleTextEditors.find((candidate) => candidate.document.uri.toString() === args.documentUri)
        : vscode.window.activeTextEditor;
    if (args.key && args.documentUri) {
        return {
            key: args.key,
            documentUri: args.documentUri,
            restorePosition: args.line !== undefined && args.character !== undefined
                ? new vscode.Position(args.line, args.character)
                : (editor?.selection.active ?? new vscode.Position(0, 0)),
        };
    }
    if (!editor || !isSupportedSourceDocument(editor.document)) {
        return undefined;
    }
    const match = findStaticI18nKeyAtPosition(editor.document, editor.selection.active);
    if (!match) {
        return undefined;
    }
    return {
        key: match.key,
        documentUri: editor.document.uri.toString(),
        restorePosition: editor.selection.active,
    };
}
async function hideI18nHover() {
    try {
        await vscode.commands.executeCommand("editor.action.hideHover");
    }
    catch {
        // Some VS Code-compatible hosts do not expose this builtin command.
    }
}
async function restoreEditorPosition(documentUri, position) {
    const document = await vscode.workspace.openTextDocument(vscode.Uri.parse(documentUri));
    const editor = await vscode.window.showTextDocument(document, {
        preserveFocus: false,
    });
    const selection = new vscode.Selection(position, position);
    editor.selection = selection;
    editor.revealRange(selection, vscode.TextEditorRevealType.InCenterIfOutsideViewport);
}
function workspaceFolderFromDocumentUri(documentUri) {
    if (!documentUri) {
        return vscode.window.activeTextEditor
            ? vscode.workspace.getWorkspaceFolder(vscode.window.activeTextEditor.document.uri)
            : undefined;
    }
    return vscode.workspace.getWorkspaceFolder(vscode.Uri.parse(documentUri));
}
function spanishLocalePath(root) {
    return node_path_1.default.join(root, "src", "i18n", "locale", "es.ts");
}
async function readCurrentSpanishValue(localePath, key) {
    const source = await (0, promises_1.readFile)(localePath, "utf8");
    const entry = findTranslationEntry(source, key);
    return entry?.value;
}
async function upsertSpanishTranslation(localePath, key, value) {
    const source = await (0, promises_1.readFile)(localePath, "utf8");
    const span = findExportedObjectSpan(source);
    if (!span) {
        throw new Error("No se encontro export const es = { ... } en src/i18n/locale/es.ts.");
    }
    const nextEntry = `    '${key}': '${escapeSingleQuotedString(value)}',`;
    const currentEntry = findTranslationEntry(source, key);
    if (currentEntry) {
        if (currentEntry.value === null) {
            throw new Error("La traduccion existe, pero no es un string editable.");
        }
        const nextSource = `${source.slice(0, currentEntry.entryStart)}${nextEntry}${source.slice(currentEntry.entryEnd)}`;
        await (0, promises_1.writeFile)(localePath, nextSource, "utf8");
        return;
    }
    const beforeClose = source.slice(0, span.close).trimEnd();
    const suffix = source.slice(span.close);
    const separator = beforeClose.endsWith("{") ? "\n" : "\n";
    await (0, promises_1.writeFile)(localePath, `${beforeClose}${separator}${nextEntry}\n${suffix}`, "utf8");
}
function findTranslationEntry(source, key) {
    const span = findExportedObjectSpan(source);
    if (!span) {
        return undefined;
    }
    const body = source.slice(span.open + 1, span.close);
    const entries = splitTopLevelEntries(body, span.open + 1);
    for (const entry of entries) {
        const parsed = parseTranslationEntry(source, entry.start, entry.end);
        if (parsed?.key === key) {
            return parsed;
        }
    }
    return undefined;
}
function parseTranslationEntry(source, start, end) {
    const raw = source.slice(start, end);
    const trimmedStartOffset = raw.length - raw.trimStart().length;
    const withoutLeadingComments = stripLeadingComments(raw.trimStart());
    const commentOffset = raw.trimStart().length - withoutLeadingComments.length;
    const normalized = withoutLeadingComments
        .trimEnd()
        .replace(/,$/, "")
        .trimEnd();
    const match = /^(['"])([^'"]+)\1\s*:\s*(['"])((?:\\.|(?!\3).)*)\3\s*$/s.exec(normalized);
    if (match) {
        return {
            key: match[2],
            value: unescapeStringLiteral(match[4]),
            entryStart: start + trimmedStartOffset + commentOffset,
            entryEnd: end,
        };
    }
    const keyOnly = /^(['"])([^'"]+)\1\s*:/.exec(normalized);
    if (keyOnly) {
        return {
            key: keyOnly[2],
            value: null,
            entryStart: start + trimmedStartOffset + commentOffset,
            entryEnd: end,
        };
    }
    return undefined;
}
function findExportedObjectSpan(source) {
    const exportIndex = source.indexOf("export const es");
    if (exportIndex < 0) {
        return undefined;
    }
    const equalsIndex = source.indexOf("=", exportIndex);
    const open = source.indexOf("{", equalsIndex);
    if (equalsIndex < 0 || open < 0) {
        return undefined;
    }
    const close = findMatchingBrace(source, open);
    return close === undefined ? undefined : { open, close };
}
function findMatchingBrace(source, open) {
    let state = "normal";
    let depth = 0;
    for (let index = open; index < source.length; index += 1) {
        const char = source[index];
        const next = source[index + 1];
        if (state === "normal") {
            if (char === "{") {
                depth += 1;
            }
            else if (char === "}") {
                depth -= 1;
                if (depth === 0) {
                    return index;
                }
            }
            else if (char === "'") {
                state = "single";
            }
            else if (char === '"') {
                state = "double";
            }
            else if (char === "`") {
                state = "template";
            }
            else if (char === "/" && next === "/") {
                state = "line-comment";
                index += 1;
            }
            else if (char === "/" && next === "*") {
                state = "block-comment";
                index += 1;
            }
        }
        else if (state === "single") {
            if (char === "\\") {
                index += 1;
            }
            else if (char === "'") {
                state = "normal";
            }
        }
        else if (state === "double") {
            if (char === "\\") {
                index += 1;
            }
            else if (char === '"') {
                state = "normal";
            }
        }
        else if (state === "template") {
            if (char === "\\") {
                index += 1;
            }
            else if (char === "`") {
                state = "normal";
            }
        }
        else if (state === "line-comment") {
            if (char === "\n") {
                state = "normal";
            }
        }
        else if (state === "block-comment" && char === "*" && next === "/") {
            state = "normal";
            index += 1;
        }
    }
    return undefined;
}
function splitTopLevelEntries(body, absoluteStart) {
    const entries = [];
    let state = "normal";
    let parens = 0;
    let braces = 0;
    let brackets = 0;
    let start = 0;
    for (let index = 0; index < body.length; index += 1) {
        const char = body[index];
        const next = body[index + 1];
        if (state === "normal") {
            if (char === "," &&
                parens === 0 &&
                braces === 0 &&
                brackets === 0) {
                entries.push({
                    start: absoluteStart + start,
                    end: absoluteStart + index + 1,
                });
                start = index + 1;
            }
            else if (char === "(") {
                parens += 1;
            }
            else if (char === ")") {
                parens = Math.max(0, parens - 1);
            }
            else if (char === "{") {
                braces += 1;
            }
            else if (char === "}") {
                braces = Math.max(0, braces - 1);
            }
            else if (char === "[") {
                brackets += 1;
            }
            else if (char === "]") {
                brackets = Math.max(0, brackets - 1);
            }
            else if (char === "'") {
                state = "single";
            }
            else if (char === '"') {
                state = "double";
            }
            else if (char === "`") {
                state = "template";
            }
            else if (char === "/" && next === "/") {
                state = "line-comment";
                index += 1;
            }
            else if (char === "/" && next === "*") {
                state = "block-comment";
                index += 1;
            }
        }
        else if (state === "single") {
            if (char === "\\") {
                index += 1;
            }
            else if (char === "'") {
                state = "normal";
            }
        }
        else if (state === "double") {
            if (char === "\\") {
                index += 1;
            }
            else if (char === '"') {
                state = "normal";
            }
        }
        else if (state === "template") {
            if (char === "\\") {
                index += 1;
            }
            else if (char === "`") {
                state = "normal";
            }
        }
        else if (state === "line-comment") {
            if (char === "\n") {
                state = "normal";
            }
        }
        else if (state === "block-comment" && char === "*" && next === "/") {
            state = "normal";
            index += 1;
        }
    }
    entries.push({
        start: absoluteStart + start,
        end: absoluteStart + body.length,
    });
    return entries.filter((entry) => sourceSliceHasContent(body, entry.start - absoluteStart, entry.end - absoluteStart));
}
function sourceSliceHasContent(source, start, end) {
    return source.slice(start, end).trim().length > 0;
}
function stripLeadingComments(value) {
    let next = value;
    while (true) {
        const trimmed = next.trimStart();
        if (trimmed.startsWith("//")) {
            const newline = trimmed.indexOf("\n");
            if (newline < 0) {
                return "";
            }
            next = trimmed.slice(newline + 1);
            continue;
        }
        if (trimmed.startsWith("/*")) {
            const end = trimmed.indexOf("*/");
            if (end < 0) {
                return "";
            }
            next = trimmed.slice(end + 2);
            continue;
        }
        return trimmed;
    }
}
function escapeSingleQuotedString(value) {
    return value
        .replace(/\\/g, "\\\\")
        .replace(/'/g, "\\'")
        .replace(/\r/g, "\\r")
        .replace(/\n/g, "\\n");
}
function unescapeStringLiteral(value) {
    return value
        .replace(/\\n/g, "\n")
        .replace(/\\r/g, "\r")
        .replace(/\\'/g, "'")
        .replace(/\\"/g, '"')
        .replace(/\\\\/g, "\\");
}
function escapeRegExp(value) {
    return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
//# sourceMappingURL=i18nEditor.js.map