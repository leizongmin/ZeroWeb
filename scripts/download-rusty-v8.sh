#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace_root="$(cd "${script_dir}/.." && pwd)"

case "$(uname -s)" in
  Linux|Darwin)
    ;;
  *)
    echo "download-rusty-v8.sh only supports Linux and macOS." >&2
    echo "On Windows, set RUSTY_V8_ARCHIVE to the release .lib URL." >&2
    exit 1
    ;;
esac

has_archive_header() {
  local path="$1"
  [[ -f "${path}" ]] || return 1
  cmp -s <(head -c 8 "${path}") <(printf '!<arch>\n') ||
    cmp -s <(head -c 2 "${path}") <(printf '\x1f\x8b')
}

cache_root="${RUSTY_V8_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/zero-web/rusty_v8}"
local_archive="${RUSTY_V8_ARCHIVE:-${workspace_root}/.cargo/rusty_v8/archive}"
if [[ "${local_archive}" != /* ]]; then
  local_archive="${workspace_root}/${local_archive}"
fi

version="$(
  sed -n '/name = "v8"/{n;s/^version = "//;s/"$//;p;q;}' "${workspace_root}/Cargo.lock"
)"
if [[ -z "${version}" ]]; then
  echo "Failed to determine v8 version from Cargo.lock" >&2
  exit 1
fi

target_triple="${RUSTY_V8_TARGET:-$(rustc -vV | sed -n 's/^host: //p')}"
if [[ -z "${target_triple}" ]]; then
  echo "Failed to determine host target triple" >&2
  exit 1
fi

profile="release"
case "${V8_FORCE_DEBUG:-}" in
  1|true|yes)
    profile="debug"
    ;;
esac

if [[ "${target_triple}" == *windows* ]]; then
  archive_name="rusty_v8_${profile}_${target_triple}.lib.gz"
else
  archive_name="librusty_v8_${profile}_${target_triple}.a.gz"
fi

cache_archive="${cache_root}/v${version}/${archive_name}"
cache_tmp="${cache_archive}.tmp"
mkdir -p "$(dirname "${cache_archive}")" "$(dirname "${local_archive}")"

if [[ -f "${cache_archive}" ]] && ! has_archive_header "${cache_archive}"; then
  echo "Removing invalid cached archive ${cache_archive}" >&2
  rm -f "${cache_archive}"
fi

if [[ -f "${cache_tmp}" ]] && ! has_archive_header "${cache_tmp}"; then
  echo "Removing invalid partial archive ${cache_tmp}" >&2
  rm -f "${cache_tmp}"
fi

if [[ -s "${cache_archive}" ]]; then
  ln -sfn "${cache_archive}" "${local_archive}"
  echo "rusty_v8 archive already cached: ${cache_archive}"
  exit 0
fi

base_url="${RUSTY_V8_MIRROR:-https://github.com/denoland/rusty_v8/releases/download}"
download_url="${base_url}/v${version}/${archive_name}"

if [[ -f "${cache_tmp}" ]]; then
  echo "Resuming download: ${download_url}"
else
  echo "Downloading: ${download_url}"
fi

curl -fL -C - --progress-bar -o "${cache_tmp}" "${download_url}"
mv "${cache_tmp}" "${cache_archive}"
ln -sfn "${cache_archive}" "${local_archive}"

echo "rusty_v8 archive ready at ${cache_archive}"
