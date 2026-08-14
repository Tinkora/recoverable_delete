#!/bin/sh
set -u

plugin_root=${PLUGIN_ROOT:-$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)}

for candidate in \
  "${RECOVERABLE_DELETE_BIN:-}" \
  "$plugin_root/bin/recoverable-delete" \
  "$plugin_root/bin/recoverable_delete"
do
  if [ -n "$candidate" ] && [ -x "$candidate" ]; then
    exec "$candidate" hook
  fi
done

printf '%s\n' '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"Recoverable Delete policy binary is unavailable. The matched tool call was blocked instead of failing open."}}'
