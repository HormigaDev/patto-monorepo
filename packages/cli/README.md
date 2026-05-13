# @patto/cli

Official CLI for Patto Bot Template projects.

`@patto/cli` is the user-facing wrapper for Patto tooling. It handles
scaffolding in TypeScript/Node and delegates heavy project analysis to the
native Rust core distributed through platform-specific optional dependencies.

Everything in this package is licensed under `AGPL-3.0-only`.

## Installation

```bash
pnpm add -g @patto/cli
```

or:

```bash
npm install -g @patto/cli
```

After installation:

```bash
patto --help
```

## Platform Support

`@patto/cli` installs the native core as optional dependencies. Supported
platforms today:

- Linux x64: `@patto/cli-core-linux-x64`
- Linux arm64: `@patto/cli-core-linux-arm64`
- Windows x64: `@patto/cli-core-win32-x64`

If your platform is not supported yet, the CLI exits with a clear error telling
you that no compatible native binary is available.

## Project Root

Commands that analyze a bot project accept `--root`:

```bash
patto check --root /path/to/patto-bot
```

If omitted, the current working directory is used.

## Scaffolding

Scaffolding is handled directly by the Node wrapper.

### Command

By default, command scaffolding creates a split command:

```bash
patto generate command info/ping
```

Creates:

```text
src/definitions/info/ping.definition.ts
src/commands/info/ping.command.ts
```

Create a single command file instead:

```bash
patto generate command info/ping --single-file
```

`--unified` is an alias of `--single-file`.

### Subcommand

```bash
patto generate subcommand get --parent config
```

Creates:

```text
src/commands/config/get.command.ts
```

### Subcommand Group

```bash
patto generate subcommand-group set --parent server --group config
```

Creates:

```text
src/commands/server/config/set.command.ts
```

### Definition

```bash
patto generate definition help
```

For subcommand definitions:

```bash
patto generate definition get --kind subcommand --parent config
```

For subcommand-group definitions:

```bash
patto generate definition set --kind subcommand-group --parent server --group config
```

### Plugin

```bash
patto generate plugin audit-log --scope deep-folder --folder moderation
```

Creates:

```text
src/plugins/audit-log.plugin.ts
```

and registers it in:

```text
src/config/plugins.config.ts
```

For `PluginScope.Specified`, provide target commands:

```bash
patto generate plugin review-gate --scope specified --commands info/about,admin/ban
```

Skip automatic registration:

```bash
patto generate plugin audit-log --no-register
```

### Generate Aliases

All generate commands can use aliases:

```bash
patto g command ping
patto scaffold command ping
```

## Analysis Commands

These commands call the native Rust core.

### scan

Indexes the project and writes `.patto/index.json`.

```bash
patto scan --root /path/to/bot
```

### lint

Runs Patto static rules over commands, definitions, plugins and project
conventions.

```bash
patto lint --root /path/to/bot
```

### doctor

Checks project health: runtime, dependencies, scripts, env files, tsconfig,
Patto config, sharding/Redis and build output.

```bash
patto doctor --root /path/to/bot
```

### check

Runs `scan + lint + doctor`. This is the recommended command for CI and editor
integrations.

```bash
patto check --root /path/to/bot
```

## Human Output

By default, diagnostics are rendered for humans:

```text
src/config/plugins.config.ts:45:15 WARNING plugin-specified-commands
  PluginScope.Specified no tiene una lista de commands válida.
  45 | //     scope: PluginScope.Specified,
     |               ^^^^^^^^^^^^^^^^^^^^^
  hint: Agrega commands: [MiCommand] cuando uses PluginScope.Specified.
```

Severity colors:

- error: red
- warning: yellow/orange
- info: blue

## JSON Output

Use `--json` to print the raw JSON returned by the Rust core:

```bash
patto check --root /path/to/bot --json
```

Diagnostics include:

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

## Stdin API

The CLI exposes a structured API for extensions and other tools:

```bash
printf '{"command":"check","root":"/path/to/bot","lang":"es"}' | patto core --stdin
```

Response shape:

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

Supported `command` values:

- `scan`
- `lint`
- `doctor`
- `check`

## Configuration

Patto projects use:

```text
.patto/config.json
```

Minimum config:

```json
{
  "schemaVersion": 1,
  "lang": "es"
}
```

Lint rules can be configured with severities:

```json
{
  "lint-rules": {
    "duplicate-commands": "error",
    "invalid-command-names": "warning",
    "ghost-parent-mix": "off"
  }
}
```

Supported severities:

- `off`
- `info`
- `warning`
- `error`

## Development

From the monorepo root:

```bash
pnpm install
pnpm --filter @patto/cli build
pnpm --filter @patto/cli dev -- --help
```

Build native binaries:

```bash
pnpm build:core
```

## License

`@patto/cli` is licensed under `AGPL-3.0-only`.

The native core packages consumed by this CLI are also licensed under
`AGPL-3.0-only`.
