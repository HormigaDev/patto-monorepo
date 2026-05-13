# patto-cli-core

Native Rust core for Patto CLI.

`patto-cli-core` contains the fast project analysis engine used by `@patto/cli`
and the Patto VS Code extension. It is designed to scan large Patto Bot
Template projects faster than a pure Node implementation and to emit
structured diagnostics suitable for terminals, CI and editors.

This crate and its binary output are licensed under `AGPL-3.0-only`.

## Binary

The binary name is:

```text
patto-core
```

Run locally:

```bash
cargo run -- check --root /path/to/bot --json
```

Build:

```bash
cargo build --release
```

## Commands

All commands accept:

```text
--root <path>   Project root. Defaults to current directory.
--json          Print structured JSON.
--lang <lang>   Global option. Defaults to auto.
```

`--lang` is a global Clap option, so it is passed before the subcommand:

```bash
patto-core --lang es lint --root /path/to/bot --json
```

### scan

Scans a Patto project and writes:

```text
.patto/index.json
```

Example:

```bash
patto-core scan --root /path/to/bot --json
```

The scanner indexes:

- project config
- package metadata
- important Patto folders
- command files
- definitions
- decorators
- arguments
- command summary counts

It excludes common heavy/generated folders such as:

- `.git`
- `.patto`
- `node_modules`
- `dist`
- `build`
- `target`
- `coverage`

### lint

Runs static Patto rules over the project.

```bash
patto-core lint --root /path/to/bot --json
```

Current rule set:

- `duplicate-commands`
- `duplicate-aliases`
- `invalid-command-names`
- `unknown-command-files`
- `decorated-base-command`
- `missing-run-method`
- `subcommand-consistency`
- `ghost-parent-mix`
- `invalid-arguments`
- `command-folder-convention`
- `broken-alias-imports`
- `plugin-specified-commands`
- `sharding-redis-config`
- `component-handler-methods`

Rule severities are read from `.patto/config.json`:

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

`warn` is accepted as an alias of `warning`.

### doctor

Checks project and environment health.

```bash
patto-core doctor --root /path/to/bot --json
```

Checks include:

- runtime
- `package.json`
- env files and required env vars
- `tsconfig.json`
- `.patto/config.json`
- sharding/Redis configuration
- build output

### check

Runs the full validation pipeline:

```text
scan + lint + doctor
```

Example:

```bash
patto-core check --root /path/to/bot --json
```

This is the recommended command for CI and editor integrations.

## Output Contract

JSON outputs use camelCase and include a top-level `diagnostics` array.

Diagnostic shape:

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

Diagnostic levels:

- `error`
- `warning`
- `info`

Exit codes:

- `0`: command completed without error diagnostics.
- `1`: command completed and found error diagnostics.
- `2`: unexpected runtime failure.

## Project Assumptions

The core targets Patto Bot Template projects with a structure similar to:

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

Minimum `.patto/config.json`:

```json
{
  "schemaVersion": 1,
  "lang": "es"
}
```

Currently, Spanish (`es`) is the mature language target.

## Distribution

The monorepo wraps this core in platform-specific npm packages:

- `@patto/cli-core-linux-x64`
- `@patto/cli-core-linux-arm64`
- `@patto/cli-core-win32-x64`

The root script:

```bash
pnpm build:core
```

runs `build-cli-core.sh`, builds release binaries and copies them into the
matching package folders.

## Development

Run tests:

```bash
cargo test
```

Format:

```bash
cargo fmt
```

Lint:

```bash
cargo clippy
```

## License

`patto-cli-core` is licensed under `AGPL-3.0-only`.

The npm packages that distribute its compiled binaries are also licensed under
`AGPL-3.0-only`.
