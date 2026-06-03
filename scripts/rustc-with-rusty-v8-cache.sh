#!/usr/bin/env bash

set -euo pipefail

real_rustc="$1"
shift

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace_root="$(cd "${script_dir}/.." && pwd)"
local_archive="${RUSTY_V8_ARCHIVE:-}"
log_file=""
curl_pid=""
heartbeat_pid=""

log_message() {
  local message="$1"
  local ts
  ts="$(date '+%Y-%m-%d %H:%M:%S')"
  if [[ -n "${log_file}" ]]; then
    printf '[zero-web/rusty_v8] %s %s\n' "${ts}" "${message}" >> "${log_file}"
  fi
  printf '[zero-web/rusty_v8] %s %s\n' "${ts}" "${message}" >&2
  if [[ ! -t 2 && -w /dev/tty ]]; then
    printf '[zero-web/rusty_v8] %s %s\n' "${ts}" "${message}" > /dev/tty 2>/dev/null || true
  fi
}

format_bytes() {
  local bytes="$1"
  if command -v numfmt >/dev/null 2>&1; then
    numfmt --to=iec-i --suffix=B "${bytes}"
  else
    printf '%sB' "${bytes}"
  fi
}

has_tty_sink() {
  [[ -w /dev/tty ]]
}

curl_stderr_args() {
  if has_tty_sink; then
    printf '%s\n' "--stderr" "/dev/tty"
  fi
}

report_download_heartbeat() {
  local pid="$1"
  local path="$2"
  local label="$3"
  local last_size="-1"

  while kill -0 "${pid}" 2>/dev/null; do
    local current_size="0"
    if [[ -f "${path}" ]]; then
      current_size="$(wc -c < "${path}" 2>/dev/null || printf '0')"
    fi

    if [[ "${current_size}" != "${last_size}" ]]; then
      log_message "${label}: downloaded $(format_bytes "${current_size}") so far"
      last_size="${current_size}"
    else
      log_message "${label}: waiting for more data, still at $(format_bytes "${current_size}")"
    fi

    sleep 5
  done
}

cleanup_download_children() {
  if [[ -n "${heartbeat_pid}" ]]; then
    kill "${heartbeat_pid}" 2>/dev/null || true
    wait "${heartbeat_pid}" 2>/dev/null || true
    heartbeat_pid=""
  fi

  if [[ -n "${curl_pid}" ]]; then
    kill "${curl_pid}" 2>/dev/null || true
    wait "${curl_pid}" 2>/dev/null || true
    curl_pid=""
  fi
}

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
  log_file="${ZERO_WEB_BUILD_LOG:-${cache_root}/download.log}"
  mkdir -p "$(dirname "${log_file}")"
  log_message "Checking rusty_v8 cache under ${cache_root}"

  exec 200>"${cache_root}/.download.lock"
  if ! flock -n 200; then
    log_message "Another process is downloading rusty_v8; waiting for ${cache_root}/.download.lock"
    flock 200
  fi

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
    log_message "Detecting host target triple for rusty_v8 archive"
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
  log_message "Resolved rusty_v8 archive v${version} for ${target_triple}: ${archive_name}"

  if [[ -f "${cache_archive}" ]] && ! has_archive_header "${cache_archive}"; then
    log_message "Removing invalid cached archive ${cache_archive}"
    rm -f "${cache_archive}"
  fi

  if [[ -f "${cache_tmp}" ]] && ! has_archive_header "${cache_tmp}"; then
    log_message "Removing invalid partial archive ${cache_tmp}"
    rm -f "${cache_tmp}"
  fi

  if [[ ! -s "${cache_archive}" ]]; then
    base_url="${RUSTY_V8_MIRROR:-https://github.com/denoland/rusty_v8/releases/download}"
    download_url="${base_url}/v${version}/${archive_name}"
    mapfile -t curl_stderr < <(curl_stderr_args)

    trap 'cleanup_download_children; exit 130' INT TERM
    trap 'cleanup_download_children' EXIT

    if [[ -f "${cache_tmp}" ]]; then
      log_message "Resuming download ${download_url}"
    else
      log_message "Downloading ${download_url}"
    fi

    log_message "Started curl for ${archive_name}; waiting for first bytes"
    if has_tty_sink; then
      curl -fL -C - --progress-bar "${curl_stderr[@]}" -o "${cache_tmp}" "${download_url}"
    else
      curl -fL -C - --progress-bar "${curl_stderr[@]}" -o "${cache_tmp}" "${download_url}" &
      curl_pid="$!"
      report_download_heartbeat "${curl_pid}" "${cache_tmp}" "${archive_name}" &
      heartbeat_pid="$!"

      curl_status=0
      if ! wait "${curl_pid}"; then
        curl_status=$?
      fi
      cleanup_download_children
      if [[ "${curl_status}" -ne 0 ]]; then
        exit "${curl_status}"
      fi
    fi

    trap - INT TERM EXIT
    mv "${cache_tmp}" "${cache_archive}"
    log_message "Cached rusty_v8 archive at ${cache_archive}"
  else
    log_message "Using cached rusty_v8 archive at ${cache_archive}"
  fi

  ln -sfn "${cache_archive}" "${local_archive}"
fi

exec "${real_rustc}" "$@"
