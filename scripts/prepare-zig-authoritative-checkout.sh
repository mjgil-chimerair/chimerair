#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <destination>" >&2
  exit 1
fi

destination="$1"
repo_url="${CHIMERA_ZIG_GIT_URL:-}"
repo_ref="${CHIMERA_ZIG_GIT_REF:-}"
repo_token="${CHIMERA_ZIG_GIT_TOKEN:-}"
repo_depth="${CHIMERA_ZIG_GIT_DEPTH:-1}"

if [[ -z "$repo_url" ]]; then
  echo "ERROR: CHIMERA_ZIG_GIT_URL is required to prepare an authoritative Zig checkout" >&2
  exit 1
fi

clone_url="$repo_url"
if [[ -n "$repo_token" && "$repo_url" =~ ^https:// ]]; then
  clone_url="https://x-access-token:${repo_token}@${repo_url#https://}"
fi

rm -rf "$destination"
git clone --depth "$repo_depth" "$clone_url" "$destination" >/dev/null 2>&1

if [[ -n "$repo_ref" ]]; then
  git -C "$destination" checkout "$repo_ref" >/dev/null 2>&1
fi

if [[ ! -d "$destination/.git" ]]; then
  echo "ERROR: cloned Zig checkout is missing .git metadata: $destination" >&2
  exit 1
fi
