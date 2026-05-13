# Patto

Editor diagnostics for Patto Bot Template projects.

The extension calls the Patto CLI through its structured stdin/stdout API:

```bash
patto core --stdin
```

If `patto` is not available, the extension shows setup instructions. It does
not download packages by itself.

## Features

- Shows Patto lint/check diagnostics directly in TypeScript files.
- Uses the same `@patto/cli` API that powers the terminal workflow.
- Uses the `patto` command from your system `PATH`.
- Runs an initial check when a Patto workspace opens.
- Refreshes diagnostics after relevant files change or are saved.

## Requirements

- VS Code 1.90 or newer.
- Node.js 18 or newer.
- `@patto/cli` available as the `patto` command in your `PATH`.
- A Patto Bot Template project.

## CLI Setup

Make `@patto/cli` available with your preferred Node package manager and ensure
the `patto` command is available in your `PATH`.

## Settings

- `patto.diagnosticsCommand`: `check` by default.
- `patto.runDiagnosticsOnOpen`: run an initial check when the workspace opens.
- `patto.runDiagnosticsOnChange`: refresh diagnostics after file edits.
- `patto.runDiagnosticsOnSave`: refresh diagnostics after saves.
- `patto.diagnosticsDebounceMs`: debounce delay for change-triggered diagnostics.
- `patto.cliPath`: optional explicit CLI path for local development.
