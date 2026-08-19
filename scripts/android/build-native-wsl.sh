#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 <source-root> <output-dir> <abi>" >&2
  exit 2
fi

source_root=$(realpath "$1")
output_dir=$(realpath -m "$2")
abi=$3

: "${ZERO_V8_SOURCE:?ZERO_V8_SOURCE must point to a recursive rusty_v8 150.2.0 checkout}"
: "${ZERO_CHROMIUM_CLANG:?ZERO_CHROMIUM_CLANG must point to Chromium clang 23}"
: "${ZERO_ANDROID_NDK:?ZERO_ANDROID_NDK must point to Linux Android NDK r30}"
: "${LIBCLANG_PATH:?LIBCLANG_PATH must point to libclang 19 or newer}"

v8_source=$(realpath "$ZERO_V8_SOURCE")
clang_root=$(realpath "$ZERO_CHROMIUM_CLANG")
ndk_root=$(realpath "$ZERO_ANDROID_NDK")
workspace=${ZERO_ANDROID_WSL_WORKSPACE:-"$HOME/.cache/zeroweb/android-native-workspace"}
target_dir=${ZERO_ANDROID_WSL_TARGET_DIR:-"$HOME/.cache/zeroweb/android-native-target"}
patch_file="$source_root/scripts/android/patches/rusty-v8-android-bindgen.patch"

[[ "$workspace" == "$HOME/.cache/zeroweb/"* ]] || { echo "ZERO_ANDROID_WSL_WORKSPACE must stay under $HOME/.cache/zeroweb" >&2; exit 2; }
[[ -f "$v8_source/Cargo.toml" ]] || { echo "invalid ZERO_V8_SOURCE: $v8_source" >&2; exit 2; }
grep -qx 'version = "150.2.0"' "$v8_source/Cargo.toml" || { echo "ZERO_V8_SOURCE must be rusty_v8 150.2.0" >&2; exit 2; }
[[ -x "$clang_root/bin/clang++" ]] || { echo "invalid ZERO_CHROMIUM_CLANG: $clang_root" >&2; exit 2; }
[[ -x "$ndk_root/toolchains/llvm/prebuilt/linux-x86_64/bin/clang" ]] || { echo "invalid ZERO_ANDROID_NDK: $ndk_root" >&2; exit 2; }

if ! git -C "$v8_source" apply --reverse --check "$patch_file" >/dev/null 2>&1; then
  git -C "$v8_source" apply "$patch_file"
fi

rm -rf "$v8_source/third_party/android_ndk" "$v8_source/third_party/android_toolchain/ndk"
mkdir -p "$v8_source/third_party/android_toolchain"
ln -s "$ndk_root" "$v8_source/third_party/android_ndk"
ln -s "$ndk_root" "$v8_source/third_party/android_toolchain/ndk"

rm -rf "$workspace"
mkdir -p "$workspace" "$output_dir"
tar \
  --exclude=.git \
  --exclude=target \
  --exclude=.gradle \
  --exclude=apps/android-browser/app/build \
  -C "$source_root" -cf - . | tar -C "$workspace" -xf -

export ANDROID_HOME="$(dirname "$(dirname "$ndk_root")")"
export ANDROID_NDK_HOME="$ndk_root"
export CLANG_BASE_PATH="$clang_root"
export CARGO_TARGET_DIR="$target_dir"
export V8_FROM_SOURCE=1
export EXTRA_GN_ARGS="android_ndk_version=30"

cd "$workspace"
cargo ndk -P 26 -t "$abi" -o "$output_dir" build --release \
  --config "patch.crates-io.v8.path=\"$v8_source\"" \
  -p zero-android-browser --features android-renderer
