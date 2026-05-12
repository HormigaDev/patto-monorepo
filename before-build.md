# Nota: linkers necesarios antes de compilar binarios Windows

Este proyecto compila binarios Rust para varias plataformas. Antes de ejecutar `cargo build --target ...`, no basta con instalar el target de Rust con `rustup`: también debe existir un linker compatible con la arquitectura de destino.

## Targets Windows soportados actualmente desde Linux

Para publicar binarios npm con nombres como `win32-x64`, usa este target Rust:

| Paquete npm / plataforma | Target Rust recomendado      | Toolchain/linker  |
| ------------------------ | ---------------------------- | ----------------- |
| `win32-x64`              | `x86_64-pc-windows-gnu`      | MinGW-w64         |

> Nota: `win32-arm64` queda fuera del flujo actual por baja prioridad y porque desde Linux requiere `llvm-mingw` con sysroot ARM64. Se puede agregar más adelante como `aarch64-pc-windows-gnullvm`.

## Instalar targets Rust

```bash
rustup target add x86_64-pc-windows-gnu
```

## Instalar linker para Windows x64

En Ubuntu/Debian:

```bash
sudo apt update
sudo apt install -y mingw-w64 gcc-mingw-w64-x86-64
```

Esto instala herramientas como:

```bash
x86_64-w64-mingw32-gcc
x86_64-w64-mingw32-gcc-ar
```

Verifica que existan estas herramientas:

```bash
which x86_64-w64-mingw32-gcc
```

## Configuración `.cargo/config.toml`

Crea o edita este archivo:

```toml
# .cargo/config.toml

[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-gcc-ar"
```

## Comandos de build

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

Los binarios quedan en:

```txt
target/x86_64-pc-windows-gnu/release/
```

En Windows el ejecutable tendrá extensión `.exe`.

## Errores comunes

### `file in wrong format`

Significa que Cargo está intentando linkear objetos de una arquitectura usando un linker de otra arquitectura.

Ejemplo típico:

```txt
host:   x86_64 Linux
target: aarch64 Windows
linker: x86_64 linker
```

Solución: configurar el linker correcto en `.cargo/config.toml`.

### `linker cc failed`

Normalmente significa que Cargo está usando el linker por defecto del sistema (`cc`) en vez del linker del target.

Solución: agregar una sección `[target...]` en `.cargo/config.toml`.

### `linker not found`

Significa que el ejecutable configurado como linker no está instalado o no está en el `PATH`.

Solución: instalar el paquete correspondiente o exportar el `PATH` correcto antes de compilar.
