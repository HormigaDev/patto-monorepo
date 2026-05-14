# @patto/cli

CLI oficial para proyectos de plantillas de [Patto Bot](https://github.com/HormigaDev/patto-bot-template).

`@patto/cli` es el wrapper orientado al usuario para las herramientas de Patto. Maneja
la generación de estructuras en TypeScript/Node y delega el análisis pesado del proyecto al
núcleo nativo en Rust distribuido mediante dependencias opcionales específicas de cada plataforma.

Todo el contenido de este paquete está licenciado bajo `AGPL-3.0-only`.

## Instalación

```bash
pnpm add -g @patto/cli
```

o:

```bash
npm install -g @patto/cli
```

Después de la instalación:

```bash
patto --help
```

## Soporte de plataformas

`@patto/cli` instala el núcleo nativo como dependencias opcionales. Plataformas compatibles actualmente:

- Linux x64: `@patto/cli-core-linux-x64`
- Linux arm64: `@patto/cli-core-linux-arm64`
- Windows x64: `@patto/cli-core-win32-x64`

Si tu plataforma aún no es compatible, la CLI finalizará mostrando un error claro indicando
que no hay un binario nativo compatible disponible.

## Inicialización de proyectos

### init

Crea un nuevo proyecto Patto Bot Template clonando el repositorio oficial.

```bash
# Modo interactivo (pregunta nombre y descripción)
patto init

# Con nombre como argumento posicional
patto init mi-bot-discord

# Totalmente no interactivo (listo para scripts)
patto init --name "Mi Bot" --description "Un bot de Discord"

# Combinado
patto init mi-bot --description "Un bot genial"
```

El comando:

1. Clona el repositorio via `git clone` si Git está disponible
2. Si Git no está instalado, descarga la última release como ZIP desde GitHub
3. Limpia el historial de git del template
4. Actualiza `package.json` con el nombre (kebab-case) y descripción
5. Inicializa un nuevo repositorio Git con commit inicial

La carpeta se crea en el directorio actual con el nombre tal como se ingresó (sin espacios).
El `name` en `package.json` se convierte automáticamente a kebab-case.

---

## Raíz del proyecto

Los comandos que analizan un proyecto de bot aceptan `--root`:

```bash
patto check --root /path/to/patto-bot
```

Si se omite, se utilizará el directorio de trabajo actual.

## Generación de estructuras

La generación de estructuras es manejada directamente por el wrapper de Node.

### Comando

Por defecto, la generación de comandos crea un comando dividido:

```bash
patto generate command info/ping
```

Crea:

```text
src/definitions/info/ping.definition.ts
src/commands/info/ping.command.ts
```

Para crear un único archivo de comando:

```bash
patto generate command info/ping --single-file
```

`--unified` es un alias de `--single-file`.

### Subcomando

```bash
patto generate subcommand get --parent config
```

Crea:

```text
src/commands/config/get.command.ts
```

### Grupo de subcomandos

```bash
patto generate subcommand-group set --parent server --group config
```

Crea:

```text
src/commands/server/config/set.command.ts
```

### Definición

```bash
patto generate definition help
```

Para definiciones de subcomandos:

```bash
patto generate definition get --kind subcommand --parent config
```

Para definiciones de grupos de subcomandos:

```bash
patto generate definition set --kind subcommand-group --parent server --group config
```

### Plugin

```bash
patto generate plugin audit-log --scope deep-folder --folder moderation
```

Crea:

```text
src/plugins/audit-log.plugin.ts
```

y lo registra en:

```text
src/config/plugins.config.ts
```

Para `PluginScope.Specified`, proporciona los comandos objetivo:

```bash
patto generate plugin review-gate --scope specified --commands info/about,admin/ban
```

Omitir el registro automático:

```bash
patto generate plugin audit-log --no-register
```

### Alias de generación

Todos los comandos de generación pueden usar alias:

```bash
patto g command ping
patto scaffold command ping
```

## Comandos de análisis

Estos comandos llaman al núcleo nativo en Rust.

### scan

Indexa el proyecto y escribe `.patto/index.json`.

```bash
patto scan --root /path/to/bot
```

### lint

Ejecuta las reglas estáticas de Patto sobre comandos, definiciones, plugins y convenciones del proyecto. Si `features.i18n` está activo, también valida keys usadas con `this.t(...)`, keys dinámicas y paridad de archivos en `src/i18n/locale`.

```bash
patto lint --root /path/to/bot
```

### doctor

Verifica la salud del proyecto: runtime, dependencias, scripts, archivos env, tsconfig,
configuración de Patto, features como i18n, sharding/Redis y salida de compilación.

```bash
patto doctor --root /path/to/bot
```

### check

Ejecuta `scan + lint + doctor`. Este es el comando recomendado para CI e integraciones de editores.

```bash
patto check --root /path/to/bot
```

### format-i18n

Ordena alfabeticamente las keys de los archivos `src/i18n/locale/*.ts` que exportan `export const <locale> = { ... }`. Es util para mantener estable el diff cuando la extension agrega traducciones inline.

```bash
patto format-i18n --root /path/to/bot
```

## Salida legible para humanos

Por defecto, los diagnósticos se muestran en un formato legible para humanos:

```text
src/config/plugins.config.ts:45:15 WARNING plugin-specified-commands
  PluginScope.Specified no tiene una lista de commands válida.
  45 | //     scope: PluginScope.Specified,
     |               ^^^^^^^^^^^^^^^^^^^^^
  hint: Agrega commands: [MiCommand] cuando uses PluginScope.Specified.
```

Colores de severidad:

- error: rojo
- warning: amarillo/naranja
- info: azul

## Salida JSON

Usa `--json` para imprimir el JSON sin procesar devuelto por el núcleo Rust:

```bash
patto check --root /path/to/bot --json
```

Los diagnósticos incluyen:

```json
{
    "level": "warning",
    "code": "plugin-specified-commands",
    "message": "PluginScope.Specified no tiene una lista de commands válida.",
    "file": "src/config/plugins.config.ts",
    "line": 45,
    "column": 15,
    "hint": "Agrega commands: [MiCommand] cuando uses PluginScope.Specified."
}
```

## API por stdin

La CLI expone una API estructurada para extensiones y otras herramientas:

```bash
printf '{"command":"check","root":"/path/to/bot","lang":"es"}' | patto core --stdin
```

Formato de respuesta:

```json
{
    "ok": true,
    "command": "check",
    "exitCode": 0,
    "stderr": "",
    "output": {},
    "diagnostics": []
}
```

Valores compatibles para `command`:

- `scan`
- `lint`
- `doctor`
- `check`
- `format-i18n`

## Configuración

Los proyectos Patto utilizan:

```text
.patto/config.json
```

Configuración mínima:

```json
{
    "schemaVersion": 1,
    "lang": "es"
}
```

Las reglas de lint pueden configurarse con niveles de severidad:

```json
{
    "lint-rules": {
        "duplicate-commands": "error",
        "invalid-command-names": "warning",
        "ghost-parent-mix": "off"
    }
}
```

Niveles de severidad compatibles:

- `off`
- `info`
- `warning`
- `error`

## Desarrollo

Desde la raíz del monorepo:

```bash
pnpm install
pnpm --filter @patto/cli build
pnpm --filter @patto/cli dev -- --help
```

Compilar binarios nativos:

```bash
pnpm build:core
```

## Licencia

`@patto/cli` está licenciado bajo `AGPL-3.0-only`.

Los paquetes del núcleo nativo consumidos por esta CLI también están licenciados bajo
`AGPL-3.0-only`.
