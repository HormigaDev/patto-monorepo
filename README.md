# Patto Monorepo

Patto Monorepo contiene las herramientas oficiales para trabajar con
proyectos creados con Patto Bot Template: el CLI de Node.js, el core nativo en
Rust, los paquetes binarios por plataforma y la extension de VS Code.

Todo el contenido de este repositorio esta licenciado bajo
`AGPL-3.0-only`. Esto incluye el core en Rust, el wrapper TypeScript, los
paquetes binarios publicados en npm, la extension de VS Code, scripts,
documentacion y cualquier archivo fuente o artefacto mantenido en este
monorepo. Revisa [LICENSE](./LICENSE) para el texto completo de la licencia.

## Que Hay Aqui

```text
.
├── patto-cli-core/                 # Core nativo en Rust
├── packages/
│   ├── cli/                        # Wrapper TypeScript/Node: @patto/cli
│   ├── cli-core-linux-x64/         # Binario nativo Linux x64
│   ├── cli-core-linux-arm64/       # Binario nativo Linux arm64
│   ├── cli-core-win32-x64/         # Binario nativo Windows x64
│   └── vscode-extension/           # Extension de VS Code
├── build-cli-core.sh               # Build/copia de binarios nativos
├── pnpm-workspace.yaml             # Workspace pnpm
└── package.json                    # Scripts raiz del monorepo
```

## Arquitectura

Patto CLI se divide en dos capas:

- **Wrapper TypeScript/Node (`@patto/cli`)**
  - Comandos ergonomicos para usuarios.
  - Scaffolding de comandos, subcomandos, grupos, definitions y plugins.
  - Resolucion del binario nativo correcto segun sistema operativo/CPU.
  - Formato humano para consola y JSON estructurado para integraciones.

- **Core Rust (`patto-cli-core`)**
  - Analisis rapido del proyecto.
  - Comandos pesados: `scan`, `lint`, `doctor` y `check`.
  - Salidas estructuradas con diagnostics, severidad, archivo, linea,
    columna e hints.

La extension de VS Code consume el mismo CLI mediante la API JSON, por lo que
la terminal y el editor comparten el mismo motor de diagnosticos.

## Paquetes

### `patto-cli-core`

Core nativo escrito en Rust. Expone el binario `patto-core`.

Comandos:

- `scan`: indexa el proyecto y escribe `.patto/index.json`.
- `lint`: ejecuta reglas estaticas especificas de Patto.
- `doctor`: revisa entorno, dependencias, configuracion y build.
- `check`: agregador de CI que ejecuta scan + lint + doctor.

Ejemplo desde el directorio del core:

```bash
cd patto-cli-core
cargo run -- lint --root /ruta/al/bot --json
```

### `packages/cli`

Wrapper oficial publicado como `@patto/cli`.

Comandos de scaffolding:

```bash
patto generate command info/ping
patto generate command info/ping --single-file
patto generate subcommand get --parent config
patto generate subcommand-group set --parent server --group config
patto generate definition help
patto generate plugin audit-log --scope deep-folder --folder moderation
```

Aliases:

```bash
patto g command ping
patto scaffold command ping
```

Comandos que consumen el core:

```bash
patto scan --root /ruta/al/bot
patto lint --root /ruta/al/bot
patto doctor --root /ruta/al/bot
patto check --root /ruta/al/bot
```

Salida JSON para herramientas:

```bash
patto check --root /ruta/al/bot --json
```

API por stdin/stdout para integraciones:

```bash
printf '{"command":"check","root":"/ruta/al/bot","lang":"es"}' | patto core --stdin
```

### `packages/cli-core-*`

Paquetes npm opcionales que contienen el binario nativo por plataforma.

Plataformas soportadas actualmente:

- `@patto/cli-core-linux-x64`
- `@patto/cli-core-linux-arm64`
- `@patto/cli-core-win32-x64`

`@patto/cli` los declara como `optionalDependencies`, de modo que npm/pnpm
instala solo el paquete compatible con la plataforma del usuario.

Darwin/macOS y Windows arm64 quedan fuera por ahora. Cuando se agreguen,
deben compilarse en un entorno compatible o con toolchains especificas.

### `packages/vscode-extension`

Extension de VS Code para proyectos Patto.

La extension:

- Se activa al abrir un workspace Patto.
- Ejecuta `check` inicialmente.
- Refresca diagnostics al cambiar o guardar archivos relevantes.
- Mapea severidades del core a diagnostics de VS Code:
  - `error`: error
  - `warning`: warning
  - `info`: information
- Usa el mismo CLI que la terminal mediante `patto core --stdin`.

Build:

```bash
pnpm --filter patto build
```

Empaquetado VSIX:

```bash
pnpm --filter patto package:vsix
```

## Requisitos

- Node.js 18 o superior.
- pnpm 10.
- Rust estable compatible con edition 2024.
- Toolchains nativos para compilar targets del core.

Para Windows x64 desde Linux se usa `x86_64-pc-windows-gnu`.
Para Linux se usan targets musl para obtener binarios estaticos.

## Instalacion De Dependencias

```bash
pnpm install
```

## Build

Build general del workspace:

```bash
pnpm build
```

Build del core nativo y copia de binarios a paquetes npm:

```bash
pnpm build:core
```

Build del CLI:

```bash
pnpm --filter @patto/cli build
```

Build de la extension:

```bash
pnpm --filter patto build
```

## Distribucion Del Core

El script [build-cli-core.sh](./build-cli-core.sh) compila el core en modo
release para las plataformas configuradas y copia los binarios a:

```text
packages/cli-core-linux-x64/bin/patto-core
packages/cli-core-linux-arm64/bin/patto-core
packages/cli-core-win32-x64/bin/patto-core.exe
```

Cada paquete nativo contiene:

```text
bin/<patto-core>
README.md
package.json
```

## Proyecto Patto Esperado

Las herramientas estan pensadas para proyectos basados en Patto Bot Template.
La estructura esperada incluye:

```text
src/commands
src/definitions
src/core
src/config
src/events
src/plugins
src/utils
.patto/config.json
```

Configuracion minima:

```json
{
  "schemaVersion": 1,
  "lang": "es"
}
```

Por ahora el idioma maduro es `es`.

## Diagnostics

Los diagnostics del core tienen forma estructurada:

```json
{
  "level": "warning",
  "code": "plugin-specified-commands",
  "message": "PluginScope.Specified no tiene una lista de commands valida.",
  "file": "src/config/plugins.config.ts",
  "line": 45,
  "column": 15,
  "hint": "Agrega commands: [MiCommand] cuando uses PluginScope.Specified."
}
```

Esto permite que:

- La consola muestre source frames con color.
- La extension de VS Code marque el archivo y la posicion exacta.
- Otras herramientas consuman el CLI por JSON sin parsear texto humano.

## Tests

Ejecutar tests del workspace:

```bash
pnpm test
```

Ejecutar tests del core Rust:

```bash
cd patto-cli-core
cargo test
```

## Licencia

Todo este monorepo esta bajo `AGPL-3.0-only`.

Eso incluye, sin excepcion:

- `patto-cli-core`
- `@patto/cli`
- `@patto/cli-core-linux-x64`
- `@patto/cli-core-linux-arm64`
- `@patto/cli-core-win32-x64`
- la extension de VS Code
- scripts de build
- documentacion
- configuracion del workspace
- artefactos mantenidos en el repositorio

Consulta [LICENSE](./LICENSE) para el texto completo.
