import chalk from "chalk";
import { execSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { mkdir, readdir, rename, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import ora from "ora";
import prompts from "prompts";
import AdmZip from "adm-zip";
function removeDiacritics(str) {
    return str.normalize("NFD").replace(/[̀-ͯ]/g, "");
}
function toKebabCase(str) {
    const clean = removeDiacritics(str);
    return clean
        .replace(/([A-Z]+)([A-Z][a-z])/g, "$1-$2")
        .replace(/([a-z\d])([A-Z])/g, "$1-$2")
        .toLowerCase()
        .replace(/[\s_]+/g, "-")
        .replace(/[^a-z0-9-]/g, "")
        .replace(/-+/g, "-")
        .replace(/^-|-$/g, "");
}
function isGitInstalled() {
    try {
        execSync("git --version", { stdio: "ignore" });
        return true;
    }
    catch {
        return false;
    }
}
async function fetchLatestTag() {
    const response = await fetch("https://api.github.com/repos/HormigaDev/patto-bot-template/releases/latest", {
        headers: {
            Accept: "application/vnd.github+json",
            "User-Agent": "@patto/cli",
        },
    });
    if (!response.ok) {
        throw new Error(`No se pudo obtener la ultima release: ${response.statusText}`);
    }
    const data = (await response.json());
    return data.tag_name;
}
async function downloadAndExtractZip(projectDir, tag) {
    const version = tag.replace(/^v/, "");
    const zipUrl = `https://github.com/HormigaDev/patto-bot-template/archive/refs/tags/${tag}.zip`;
    const zipPath = path.join(path.dirname(projectDir), `temp-patto-${Date.now()}.zip`);
    const response = await fetch(zipUrl);
    if (!response.ok) {
        throw new Error(`No se pudo descargar el release ${tag}: ${response.statusText}`);
    }
    const buffer = Buffer.from(await response.arrayBuffer());
    await writeFile(zipPath, buffer);
    try {
        const zip = new AdmZip(zipPath);
        zip.extractAllTo(projectDir, true);
        const extractedDir = path.join(projectDir, `patto-bot-template-${version}`);
        if (existsSync(extractedDir)) {
            const files = await readdir(extractedDir);
            for (const file of files) {
                await rename(path.join(extractedDir, file), path.join(projectDir, file));
            }
            await rm(extractedDir, { recursive: true, force: true });
        }
    }
    finally {
        await rm(zipPath, { force: true });
    }
}
export function registerInitCommand(cli) {
    cli.command("init [name]", "Inicializa un nuevo proyecto Patto Bot Template")
        .option("--name <name>", "Nombre del proyecto (alternativa al argumento posicional)")
        .option("--description <desc>", "Descripcion del proyecto")
        .action(async (nameArg, options) => {
        let projectName = nameArg ?? options.name;
        if (!projectName) {
            const response = await prompts({
                type: "text",
                name: "projectName",
                message: chalk.cyan("Nombre del proyecto (ej. MiBot):"),
                validate: (v) => v.trim().length > 0
                    ? true
                    : "El nombre no puede estar vacio",
            });
            if (!response.projectName) {
                console.log(chalk.red("Operacion cancelada."));
                process.exitCode = 1;
                return;
            }
            projectName = response.projectName;
        }
        let description = options.description;
        if (!description) {
            const response = await prompts({
                type: "text",
                name: "description",
                message: chalk.cyan("Descripcion del proyecto:"),
                initial: "Un bot creado con Patto Bot Template",
            });
            if (response.description === undefined) {
                console.log(chalk.red("Operacion cancelada."));
                process.exitCode = 1;
                return;
            }
            description =
                response.description ||
                    "Un bot creado con Patto Bot Template";
        }
        const folderName = removeDiacritics(projectName).replace(/\s+/g, "");
        const packageName = toKebabCase(projectName);
        const projectDir = path.join(process.cwd(), folderName);
        if (existsSync(projectDir)) {
            console.error(chalk.red(`El directorio "${folderName}" ya existe en ${process.cwd()}.`));
            process.exitCode = 1;
            return;
        }
        await mkdir(projectDir, { recursive: true });
        const spinner = ora();
        const hasGit = isGitInstalled();
        try {
            if (hasGit) {
                spinner.start(`Clonando repositorio en ${chalk.bold(folderName)}...`);
                execSync(`git clone --quiet https://github.com/HormigaDev/patto-bot-template.git "${folderName}"`, { stdio: "ignore" });
                spinner.succeed("Repositorio clonado.");
            }
            else {
                spinner.start("Obteniendo ultima release...");
                const tag = await fetchLatestTag();
                spinner.text = `Descargando ${chalk.bold(tag)} como ZIP...`;
                await downloadAndExtractZip(projectDir, tag);
                spinner.succeed(`Release ${chalk.bold(tag)} descargada y extraida.`);
            }
            const gitDir = path.join(projectDir, ".git");
            if (existsSync(gitDir)) {
                await rm(gitDir, { recursive: true, force: true });
            }
            const packageJsonPath = path.join(projectDir, "package.json");
            if (existsSync(packageJsonPath)) {
                const pkg = JSON.parse(readFileSync(packageJsonPath, "utf8"));
                pkg["name"] = packageName;
                pkg["description"] = description;
                pkg["version"] = "0.0.0";
                pkg["author"] = "";
                writeFileSync(packageJsonPath, JSON.stringify(pkg, null, 2), "utf8");
            }
            if (hasGit) {
                spinner.start("Inicializando repositorio Git...");
                execSync("git init", { cwd: projectDir, stdio: "ignore" });
                execSync("git add .", { cwd: projectDir, stdio: "ignore" });
                execSync('git commit -m "Initial Commit: Project Created" --no-verify', {
                    cwd: projectDir,
                    stdio: "ignore",
                });
                spinner.succeed("Repositorio Git inicializado.");
            }
            console.log(chalk.green(`\n¡Proyecto ${chalk.bold(folderName)} creado exitosamente!`));
            console.log(chalk.gray("\nPrimeros pasos:"));
            console.log(chalk.gray(`  cd ${folderName}`));
            console.log(chalk.gray("  cp .env.template .env"));
            console.log(chalk.gray("  pnpm install"));
            console.log(chalk.gray("  pnpm dev"));
            if (!hasGit) {
                console.log(chalk.yellow("\nGit no esta instalado. Inicializa el repo manualmente:"));
                console.log(chalk.gray('  git init && git add . && git commit -m "Initial Commit: Project Created"'));
            }
        }
        catch (error) {
            spinner.fail("Error al crear el proyecto.");
            console.error(chalk.red(error instanceof Error ? error.message : String(error)));
            if (existsSync(projectDir)) {
                await rm(projectDir, { recursive: true, force: true });
            }
            process.exitCode = 1;
        }
    });
}
//# sourceMappingURL=init.js.map