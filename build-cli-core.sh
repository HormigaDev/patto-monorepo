#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CORE_DIR="$ROOT_DIR/patto-cli-core"

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is required to build patto-cli-core" >&2
  exit 1
fi

require_command() {
  local command="$1"
  local target="$2"
  local install_hint="$3"

  if ! command -v "$command" >/dev/null 2>&1; then
    echo "error: $command is required to build target $target" >&2
    echo "hint: $install_hint" >&2
    exit 1
  fi
}

check_target_tools() {
  local target="$1"

  case "$target" in
    x86_64-pc-windows-gnu)
      require_command "x86_64-w64-mingw32-gcc" "$target" \
        "install mingw-w64 and gcc-mingw-w64-x86-64"
      require_command "x86_64-w64-mingw32-gcc-ar" "$target" \
        "install mingw-w64 and gcc-mingw-w64-x86-64"
      ;;
    aarch64-unknown-linux-musl)
      require_command "aarch64-linux-gnu-gcc" "$target" \
        "install an aarch64 Linux cross compiler, for example gcc-aarch64-linux-gnu"
      ;;
  esac
}

build_distribution() {
  local distribution="$1"
  local target="$2"
  local binary="$3"
  local package_dir="$ROOT_DIR/packages/cli-core-$distribution"
  local source_binary="$CORE_DIR/target/$target/release/$binary"

  echo "==> Building $distribution ($target)"

  if command -v rustup >/dev/null 2>&1; then
    rustup target add "$target"
  fi

  check_target_tools "$target"

  if [[ "$target" == *"linux-musl"* || "$target" == *"windows"* ]]; then
    (
      cd "$CORE_DIR"
      RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target "$target"
    )
  else
    (
      cd "$CORE_DIR"
      cargo build --release --target "$target"
    )
  fi

  if [[ ! -f "$source_binary" ]]; then
    echo "error: expected binary was not produced: $source_binary" >&2
    exit 1
  fi

  mkdir -p "$package_dir/bin"
  rm -f "$package_dir/bin/patto-core" "$package_dir/bin/patto-core.exe"
  cp "$source_binary" "$package_dir/bin/$binary"

  if [[ "$binary" != *.exe ]]; then
    chmod 755 "$package_dir/bin/$binary"
  fi

  echo "==> Copied $binary to packages/cli-core-$distribution/bin/"
}

build_distribution "linux-arm64" "aarch64-unknown-linux-musl" "patto-core"
build_distribution "linux-x64" "x86_64-unknown-linux-musl" "patto-core"
build_distribution "win32-x64" "x86_64-pc-windows-gnu" "patto-core.exe"

echo "All patto-core distribution binaries were built successfully."
