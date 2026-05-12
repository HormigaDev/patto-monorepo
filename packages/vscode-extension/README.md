# Patto

Editor diagnostics for Patto Bot Template projects.

The extension calls the Patto CLI through its structured stdin/stdout API:

```bash
patto core --stdin
```

If `patto` is not available, the extension shows setup instructions and lets
you configure a custom CLI path. It does not download or install packages by
itself.

## Features

- Shows Patto lint/check diagnostics directly in TypeScript files.
- Uses the same `@patto/cli` API that powers the terminal workflow.
- Detects the CLI from `PATH`, `node_modules/.bin`, or the `patto.cliPath` setting.

## Requirements

- VS Code 1.90 or newer.
- Node.js 18 or newer.
- `@patto/cli` installed globally or in the current workspace.
- A Patto Bot Template project.

## CLI Setup

Install `@patto/cli` manually using your preferred Node package manager. If your
CLI is installed in a custom location, set `patto.cliPath` in VS Code.
