#!/usr/bin/env bash

set -euo pipefail

real_rustc="$1"
shift

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace_root="$(cd "${script_dir}/.." && pwd)"
local_archive="${RUSTY_V8_ARCHIVE:-}"

has_archive_header() {
  local path="$1"
  [[ -f "${path}" ]] || return 1
  cmp -s <(head -c 8 "${path}") <(printf '!<arch>\n')
}

if [[ -n "${local_archive}" && "${local_archive}" != http:* && "${local_archive}" != https:* ]]; then
  case "$(uname -s)" in
    Linux|Darwin)
      ;;
    *)
      exec "${real_rustc}" "$@"
      ;;
  esac

  cache_root="${RUSTY_V8_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/zero-web/rusty_v8}"
  mkdir -p "${cache_root}"

  exec 200>"${cache_root}/.download.lock"
  flock 200

  version="$(
    sed -n '/name = "rusty_v8"/{n;s/^version = "//;s/"$//;p;q;}' "${workspace_root}/Cargo.lock"
  )"

  if [[ -z "${version}" ]]; then
    echo "Failed to determine rusty_v8 version from Cargo.lock" >&2
    exit 1
  fi

  target_triple=""
  prev_arg=""
  for arg in "$@"; do
    if [[ "${prev_arg}" == "--target" ]]; then
      target_triple="${arg}"
      break
    fi
    case "${arg}" in
      --target=*)
        target_triple="${arg#--target=}"
        break
        ;;
    esac
    prev_arg="${arg}"
  done

  if [[ -z "${target_triple}" ]]; then
    target_triple="$("${real_rustc}" -vV | sed -n 's/^host: //p')"
  fi

  if [[ "${target_triple}" == *windows* ]]; then
    archive_name="rusty_v8_release_${target_triple}.lib"
  else
    profile="release"
    case "${V8_FORCE_DEBUG:-}" in
      1|true|yes)
        profile="debug"
        ;;
    esac
    archive_name="librusty_v8_${profile}_${target_triple}.a"
  fi

  cache_archive="${cache_root}/v${version}/${archive_name}"
  cache_tmp="${cache_archive}.tmp"
  mkdir -p "$(dirname "${cache_archive}")" "$(dirname "${local_archive}")"

  if [[ -f "${cache_archive}" ]] && ! has_archive_header "${cache_archive}"; then
    rm -f "${cache_archive}"
  fi

  if [[ -f "${cache_tmp}" ]] && ! has_archive_header "${cache_tmp}"; then
    rm -f "${cache_tmp}"
  fi

  if [[ ! -s "${cache_archive}" ]]; then
    base_url="${RUSTY_V8_MIRROR:-https://github.com/denoland/rusty_v8/releases/download}"
    download_url="${base_url}/v${version}/${archive_name}"

    if [[ -f "${cache_tmp}" ]]; then
      echo "Resuming ${download_url}" >&2
    else
      echo "Downloading ${download_url}" >&2
    fi

    curl -fL -C - --progress-bar -o "${cache_tmp}" "${download_url}"
    mv "${cache_tmp}" "${cache_archive}"
  fi

  ln -sfn "${cache_archive}" "${local_archive}"
fi

exec "${real_rustc}" "$@"
