#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

cd "$SCRIPT_DIR"

echo "=== Chimera Runtime Sanitizer Tests ==="

bash "$SCRIPT_DIR/build.sh"

cat > "$TMPDIR/sanitizer_runner.c" <<'EOF'
#include <chimera_sanitizers.h>
#include <stdio.h>

int main(void) {
    chimera_sanitizers_init();

    if (!chimera_sanitizer_conformance_run()) {
        fprintf(stderr, "sanitizer conformance failed\n");
        return 1;
    }

    if (chimera_sanitizer_conformance_count() == 0) {
        fprintf(stderr, "no sanitizer conformance tests registered\n");
        return 2;
    }

    if (chimera_sanitizer_conformance_name(0) == NULL) {
        fprintf(stderr, "sanitizer conformance test names unavailable\n");
        return 3;
    }

    puts(chimera_sanitizers_report());
    return 0;
}
EOF

echo "Compiling sanitizer conformance runner..."
gcc -Wall -Wextra -I "$SCRIPT_DIR/include" \
    "$TMPDIR/sanitizer_runner.c" \
    "$SCRIPT_DIR/build/libchimera-rt.a" \
    -o "$TMPDIR/sanitizer_runner"

echo "Running sanitizer conformance runner..."
"$TMPDIR/sanitizer_runner"

echo "Verifying sanitizer header compiles..."
printf '#include <chimera_sanitizers.h>\n' > "$TMPDIR/header_smoke.c"
gcc -c -Wall -Wextra -fsyntax-only -I "$SCRIPT_DIR/include" "$TMPDIR/header_smoke.c"

if command -v clang >/dev/null 2>&1; then
    echo "Verifying ASan build compatibility..."
    clang -fsanitize=address -I "$SCRIPT_DIR/include" \
        -c "$SCRIPT_DIR/src/chimera_sanitizers.c" \
        -o "$TMPDIR/chimera_sanitizers_asan.o"
fi

echo "=== Runtime sanitizer tests passed ==="
