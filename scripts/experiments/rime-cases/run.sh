#!/usr/bin/env bash
# Reproduce the three Rime quick5 cases (docs/research/rime.md §4).
#
# usage: run.sh <variant> <input>...
#   variant: base   — rime-quick as shipped (derive keeps full Cangjie codes)
#            xform  — speller/algebra replaced by xform (quick codes only)
#   input:   "mir" prints candidates; "mir#143,0" selects candidates in turn
#
# env: RIME_USER_SRC  where quick5.*.yaml live (default ~/Library/Rime)
#      WITH_USERDB=1  also copy quick5.userdb (default: start with no learning)
#      N              candidates to print (default 10)
# Needs Squirrel installed (for SharedSupport and librime 1.16.0) and
# `brew install librime` (headers to build probe). Never writes to RIME_USER_SRC.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
variant="${1:?variant: base|xform}"; shift
squirrel="/Library/Input Methods/Squirrel.app/Contents"
user_src="${RIME_USER_SRC:-$HOME/Library/Rime}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

prefix="$(brew --prefix librime)"
cc -o "$work/probe" "$here/probe.c" -I"$prefix/include" -L"$prefix/lib" -lrime

mkdir -p "$work/shared" "$work/user/lua" "$work/log"
cp -R "$squirrel/SharedSupport/." "$work/shared/"
for f in quick5.schema.yaml quick5.dict.yaml quick5.supplement.dict.yaml \
         cangjie5.base.dict.yaml default.custom.yaml; do
  cp "$user_src/$f" "$work/user/"
done
if [[ "${WITH_USERDB:-0}" == 1 && -d "$user_src/quick5.userdb" ]]; then
  cp -R "$user_src/quick5.userdb" "$work/user/"
fi
cp "$here/lua/dbg.lua" "$work/user/lua/"

{
  echo "patch:"
  echo "  engine/filters/@before 0: lua_filter@*dbg"
  if [[ "$variant" == xform ]]; then
    echo "  speller/algebra:"
    echo "    - 'xform/^([^z])\\w+(\\w)\$/\$1\$2/'"
  fi
} > "$work/user/quick5.custom.yaml"

DYLD_LIBRARY_PATH="$squirrel/Frameworks" \
  "$work/probe" "$work/shared" "$work/user" quick5 "${N:-10}" "$@"
