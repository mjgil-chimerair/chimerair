#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <fixture-root>" >&2
  exit 1
fi

fixture_root="$1"

mkdir -p \
  "$fixture_root/build/stage3/bin" \
  "$fixture_root/scripts" \
  "$fixture_root/.git"
touch "$fixture_root/CMakeLists.txt"

cat >"$fixture_root/build/stage3/bin/zig" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
case "${1:-}" in
  version)
    echo "0.13.0-zigmera"
    ;;
  --help)
    cat <<'HELP'
Usage: zig [command]
  --emit-zigmera-snapshot
  --emit-zdep
  --emit-zairpack
HELP
    ;;
  *)
    exit 0
    ;;
esac
EOF
chmod +x "$fixture_root/build/stage3/bin/zig"

cat >"$fixture_root/scripts/test-zigmera.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
"${CHIMERA_ZIG_BIN:?missing CHIMERA_ZIG_BIN}" version >/dev/null
EOF
chmod +x "$fixture_root/scripts/test-zigmera.sh"
