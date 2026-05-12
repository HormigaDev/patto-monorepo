const DISCORD_NAME_MAX_LENGTH = 32;
export function parseScaffoldName(input, label) {
    const raw = input.trim();
    if (raw.length === 0) {
        throw new Error(`El nombre de ${label} no puede estar vacío.`);
    }
    if (raw.startsWith('/') || raw.includes('\\')) {
        throw new Error(`El nombre de ${label} debe ser una ruta relativa con "/" si usa carpetas.`);
    }
    const rawParts = raw.split('/').filter(Boolean);
    if (rawParts.length === 0) {
        throw new Error(`El nombre de ${label} no puede estar vacío.`);
    }
    if (rawParts.some((part) => part === '.' || part === '..')) {
        throw new Error(`El nombre de ${label} no puede contener "." ni "..".`);
    }
    const parts = rawParts.map((part) => normalizeSegment(part));
    const invalidPart = parts.find((part) => part.length === 0);
    if (invalidPart !== undefined) {
        throw new Error(`El nombre de ${label} contiene un segmento inválido.`);
    }
    const name = parts[parts.length - 1];
    validateDiscordName(name, label);
    return {
        input,
        name,
        fileBase: name,
        classBase: toPascalCase(name),
        dirs: parts.slice(0, -1),
        parts,
    };
}
export function parseDiscordName(input, label) {
    const name = normalizeSegment(input.trim());
    validateDiscordName(name, label);
    return name;
}
export function parsePathSegments(input, label) {
    const parsed = parseScaffoldName(input, label);
    return parsed.parts;
}
export function toPascalCase(value) {
    return value
        .split(/[-_]/g)
        .filter(Boolean)
        .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
        .join('');
}
export function commandClassNameFromPath(input) {
    const parsed = parseScaffoldName(input, 'comando');
    return `${parsed.classBase}Command`;
}
export function commandImportPath(input) {
    const parsed = parseScaffoldName(input, 'comando');
    return `@/commands/${[...parsed.dirs, `${parsed.fileBase}.command`].join('/')}`;
}
function normalizeSegment(value) {
    return value
        .normalize('NFD')
        .replace(/[\u0300-\u036f]/g, '')
        .toLowerCase()
        .replace(/[_\s]+/g, '-')
        .replace(/[^a-z0-9-]/g, '')
        .replace(/-+/g, '-')
        .replace(/^-|-$/g, '');
}
function validateDiscordName(name, label) {
    if (!/^[a-z0-9]+(?:[-_][a-z0-9]+)*$/.test(name)) {
        throw new Error(`El nombre de ${label} debe usar minúsculas, números, guiones o guion bajo.`);
    }
    if (name.length > DISCORD_NAME_MAX_LENGTH) {
        throw new Error(`El nombre de ${label} no puede superar ${DISCORD_NAME_MAX_LENGTH} caracteres.`);
    }
}
//# sourceMappingURL=names.js.map