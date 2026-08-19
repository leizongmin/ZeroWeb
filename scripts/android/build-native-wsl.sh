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
: "${ZERO_V8_GN:?ZERO_V8_GN must point to the Linux gn executable}"
: "${ZERO_V8_NINJA:?ZERO_V8_NINJA must point to the Linux ninja executable}"

v8_source=$(realpath "$ZERO_V8_SOURCE")
clang_root=$(realpath "$ZERO_CHROMIUM_CLANG")
ndk_root=$(realpath "$ZERO_ANDROID_NDK")
gn=$(realpath "$ZERO_V8_GN")
ninja=$(realpath "$ZERO_V8_NINJA")
workspace=${ZERO_ANDROID_WSL_WORKSPACE:-"$HOME/.cache/zeroweb/android-native-workspace"}
target_dir=${ZERO_ANDROID_WSL_TARGET_DIR:-"$HOME/.cache/zeroweb/android-native-target"}
patch_file="$source_root/scripts/android/patches/rusty-v8-android-bindgen.patch"
normalized_patch=$(mktemp)
trap 'rm -f "$normalized_patch"' EXIT
tr -d '\r' < "$patch_file" > "$normalized_patch"

[[ "$workspace" == "$HOME/.cache/zeroweb/"* ]] || { echo "ZERO_ANDROID_WSL_WORKSPACE must stay under $HOME/.cache/zeroweb" >&2; exit 2; }
[[ -f "$v8_source/Cargo.toml" ]] || { echo "invalid ZERO_V8_SOURCE: $v8_source" >&2; exit 2; }
grep -qx 'version = "150.2.0"' "$v8_source/Cargo.toml" || { echo "ZERO_V8_SOURCE must be rusty_v8 150.2.0" >&2; exit 2; }
[[ -x "$clang_root/bin/clang++" ]] || { echo "invalid ZERO_CHROMIUM_CLANG: $clang_root" >&2; exit 2; }
[[ -x "$ndk_root/toolchains/llvm/prebuilt/linux-x86_64/bin/clang" ]] || { echo "invalid ZERO_ANDROID_NDK: $ndk_root" >&2; exit 2; }
[[ -x "$gn" && -x "$ninja" ]] || { echo "ZERO_V8_GN and ZERO_V8_NINJA must be executable" >&2; exit 2; }

if ! git -C "$v8_source" apply --reverse --check "$normalized_patch" >/dev/null 2>&1; then
  git -C "$v8_source" apply "$normalized_patch"
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

case "$abi" in
  x86_64) cargo_target=x86_64-linux-android ;;
  arm64-v8a) cargo_target=aarch64-linux-android ;;
  *) echo "unsupported ABI: $abi" >&2; exit 2 ;;
esac
tool_dir="$target_dir/$cargo_target/release/ninja_gn_binaries"
mkdir -p "$tool_dir/gn" "$tool_dir/ninja"
cp "$gn" "$tool_dir/gn/gn"
cp "$ninja" "$tool_dir/ninja/ninja"

cd "$workspace"
cargo ndk -P 26 -t "$abi" -o "$output_dir" build --release \
  --config "patch.crates-io.v8.path=\"$v8_source\"" \
  -p zero-android-browser --features android-renderer

case "$abi" in
  x86_64) ndk_triple=x86_64-linux-android ;;
  arm64-v8a) ndk_triple=aarch64-linux-android ;;
esac
runtime_lib="$ndk_root/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/$ndk_triple/libc++_shared.so"
[[ -f "$runtime_lib" ]] || { echo "missing NDK C++ runtime: $runtime_lib" >&2; exit 2; }
cp "$runtime_lib" "$output_dir/$abi/libc++_shared.so"
