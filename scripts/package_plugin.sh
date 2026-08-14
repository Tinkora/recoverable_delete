#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
output_root=${1:-"$repo_root/dist"}
plugin_source="$repo_root/plugins/recoverable-delete"
package_root="$output_root/recoverable-delete"

if [ -e "$package_root" ]; then
  printf 'package destination already exists: %s\n' "$package_root" >&2
  printf 'move it to Trash before packaging again\n' >&2
  exit 1
fi

policy_binary=${RECOVERABLE_DELETE_BIN:-}
if [ -z "$policy_binary" ]; then
  cargo build --manifest-path "$repo_root/Cargo.toml" --release --locked
  policy_binary="$repo_root/target/release/recoverable-delete"
fi

if [ ! -f "$policy_binary" ]; then
  printf 'policy binary not found: %s\n' "$policy_binary" >&2
  exit 1
fi

mkdir -p "$package_root/bin"
cp -R "$plugin_source/." "$package_root/"
cp "$policy_binary" "$package_root/bin/recoverable-delete"
chmod +x "$package_root/bin/recoverable-delete"

printf '%s\n' "$package_root"
