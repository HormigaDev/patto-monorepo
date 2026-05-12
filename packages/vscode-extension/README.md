# Patto

Editor diagnostics for Patto Bot Template projects.

The extension calls the Patto CLI through its structured stdin/stdout API:

```bash
patto core --stdin
```

If `patto` is not available, the extension can install `@patto/cli` globally.
It tries `pnpm` first and falls back to `npm`.

## Features

- Shows Patto lint/check diagnostics directly in TypeScript files.
- Uses the same `@patto/cli` API that powers the terminal workflow.
- Installs the CLI on demand when it is missing.

## Requirements

- VS Code 1.90 or newer.
- Node.js 18 or newer.
- A Patto Bot Template project.
