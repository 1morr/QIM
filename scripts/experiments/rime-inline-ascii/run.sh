#!/usr/bin/env bash
# Probe librime ascii_composer inline_ascii (temporary English) key by key.
#
# usage: run.sh <step>...
#   step: a librime key sequence, e.g. anr '{Shift_L}{Release+Shift_L}' hello '{space}'
#
# env: RIME_USER_SRC  where quick5.*.yaml live (default ~/Library/Rime)
# Needs Squirrel installed (SharedSupport and its bundled librime) and
# `brew install librime` (headers to build probe). Never writes to RIME_USER_SRC.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
squirrel="/Library/Input Methods/Squirrel.app/Contents"
user_src="${RIME_USER_SRC:-$HOME/Library/Rime}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

prefix="$(brew --prefix librime)"
cc -o "$work/probe" "$here/probe.c" -I"$prefix/include" -L"$prefix/lib" -lrime

mkdir -p "$work/shared" "$work/user" "$work/log"
cp -R "$squirrel/SharedSupport/." "$work/shared/"
for f in quick5.schema.yaml quick5.dict.yaml quick5.supplement.dict.yaml \
         cangjie5.base.dict.yaml default.custom.yaml; do
  cp "$user_src/$f" "$work/user/"
done

DYLD_LIBRARY_PATH="$squirrel/Frameworks" "$work/probe" "$work/shared" "$work/user" quick5 "$@"
