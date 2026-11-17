#!/usr/bin/env bash
# Fetches the pinned official Windows Vulkan x64 `llama.cpp` release archive for OraType's
# Built-in AI (`llama-server.exe`), verifies its SHA-256 checksum, and stages `llama-server.exe`
# alongside all required runtime DLLs for Windows portable packaging and local debug builds.
#
# Usage:
#   bash scripts/fetch-llama-server-windows.sh [output_dir]
#
# Environment:
#   ORATYPE_LLAMA_ZIP_CACHE=/path/to/llama-b11177-bin-win-vulkan-x64.zip
set -euo pipefail

LLAMA_CPP_TAG="b11177"
ASSET_NAME="llama-b11177-bin-win-vulkan-x64.zip"
ASSET_URL="https://github.com/ggml-org/llama.cpp/releases/download/${LLAMA_CPP_TAG}/${ASSET_NAME}"
ASSET_SHA256="6e4e7a8cea0d2dbd0a745b55f6b42d46d8130f8df4791938802f8269dfc205d5"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="${1:-$ROOT/src-tauri/binaries/win-vulkan-x64}"
BINARIES_DIR="$ROOT/src-tauri/binaries"

say() { printf '\033[1m==>\033[0m %s\n' "$*"; }
fail() { printf '\033[31mError:\033[0m %s\n' "$*" >&2; exit 1; }

command -v sha256sum >/dev/null || fail "sha256sum is required."
command -v unzip >/dev/null || fail "unzip is required."
command -v curl >/dev/null || fail "curl is required."

TMP_DIR="$(mktemp -d /tmp/oratype-llama-win-XXXXXX)"
# The archive and its extracted copy are only needed until the files are staged.
trap 'rm -rf "$TMP_DIR"' EXIT
ZIP_PATH="$TMP_DIR/$ASSET_NAME"

if [[ -n "${ORATYPE_LLAMA_ZIP_CACHE:-}" && -f "${ORATYPE_LLAMA_ZIP_CACHE}" ]]; then
  ACTUAL_CACHE_SHA="$(sha256sum "$ORATYPE_LLAMA_ZIP_CACHE" | awk '{print $1}')"
  if [[ "$ACTUAL_CACHE_SHA" == "$ASSET_SHA256" ]]; then
    say "Reusing verified cached archive $ORATYPE_LLAMA_ZIP_CACHE"
    cp -f "$ORATYPE_LLAMA_ZIP_CACHE" "$ZIP_PATH"
  fi
fi

if [[ ! -f "$ZIP_PATH" ]]; then
  say "Downloading $ASSET_URL"
  curl -fL --retry 3 --output "$ZIP_PATH" "$ASSET_URL"
fi

ACTUAL_SHA="$(sha256sum "$ZIP_PATH" | awk '{print $1}')"
[[ "$ACTUAL_SHA" == "$ASSET_SHA256" ]] || fail "SHA-256 mismatch: expected $ASSET_SHA256, got $ACTUAL_SHA"
say "Verified SHA-256: $ACTUAL_SHA"

EXTRACT_DIR="$TMP_DIR/extracted"
mkdir -p "$EXTRACT_DIR"
unzip -qo "$ZIP_PATH" -d "$EXTRACT_DIR"

RUNTIME_FILES=(
  "llama-server.exe"
  "llama-server-impl.dll"
  "llama-common.dll"
  "llama.dll"
  "mtmd.dll"
  "ggml.dll"
  "ggml-base.dll"
  "ggml-vulkan.dll"
  "ggml-rpc.dll"
  "libomp.dll"
  "ggml-cpu-alderlake.dll"
  "ggml-cpu-cannonlake.dll"
  "ggml-cpu-cascadelake.dll"
  "ggml-cpu-cooperlake.dll"
  "ggml-cpu-haswell.dll"
  "ggml-cpu-icelake.dll"
  "ggml-cpu-ivybridge.dll"
  "ggml-cpu-piledriver.dll"
  "ggml-cpu-sandybridge.dll"
  "ggml-cpu-sapphirerapids.dll"
  "ggml-cpu-skylakex.dll"
  "ggml-cpu-sse42.dll"
  "ggml-cpu-x64.dll"
  "ggml-cpu-zen4.dll"
  "LICENSE-LLVM-OpenMP"
)

mkdir -p "$OUT_DIR" "$BINARIES_DIR"
for file in "${RUNTIME_FILES[@]}"; do
  src="$EXTRACT_DIR/$file"
  [[ -f "$src" ]] || fail "Expected runtime file missing from archive: $file"
  cp -f "$src" "$OUT_DIR/$file"
  if [[ "$file" == *.dll ]]; then
    cp -f "$src" "$BINARIES_DIR/$file"
  fi
done
cp -f "$EXTRACT_DIR/llama-server.exe" "$BINARIES_DIR/llama-server-x86_64-pc-windows-gnu.exe"

say "Staged ${#RUNTIME_FILES[@]} files to $OUT_DIR"
