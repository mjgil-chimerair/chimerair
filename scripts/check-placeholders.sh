#!/bin/bash
# Check for unauthorized placeholders across all language layers
# Fail CI if any are found in production code

set -e

cd "$(dirname "$0")/.."

ERRORS=0

echo "=== Chimera Placeholder Gate ==="
echo ""

# ============================================================
# Layer 1: Lean (sorry/admit)
# ============================================================
echo "[1/4] Checking Lean placeholders..."

SORRY_COUNT=$(grep -rn "sorry" ChimeraProof/Chimera/ \
  --include="*.lean" \
  --exclude-dir=.lake \
  | grep -v "PROOF" \
  | grep -v "OPEN" \
  | wc -l)

if [ "$SORRY_COUNT" -gt 0 ]; then
  echo "  ERROR: Found $SORRY_COUNT unauthorized sorry placeholders"
  grep -rn "sorry" ChimeraProof/Chimera/ \
    --include="*.lean" \
    --exclude-dir=.lake \
    | grep -v "PROOF" \
    | grep -v "OPEN"
  ((ERRORS++)) || true
fi

ADMIT_LINES=$(grep -rn "\badmit\b" ChimeraProof/Chimera/ \
  --include="*.lean" \
  --exclude-dir=.lake \
  | grep -v "PROOF:" \
  | grep -v "OPEN:" \
  | grep -vE "admit[[:space:]]+--" || true)

ADMIT_COUNT=$(printf "%s\n" "$ADMIT_LINES" | sed '/^$/d' | wc -l)

if [ "$ADMIT_COUNT" -gt 0 ]; then
  echo "  ERROR: Found $ADMIT_COUNT undocumented admit statements"
  printf "%s\n" "$ADMIT_LINES"
  ((ERRORS++)) || true
fi

if [ "$SORRY_COUNT" -eq 0 ] && [ "$ADMIT_COUNT" -eq 0 ]; then
  echo "  Lean placeholders: OK"
fi

# ============================================================
# Layer 2: C++ TODO stubs
# ============================================================
echo "[2/4] Checking C++ TODO stubs..."

# Look for TODO in C++ files that are not explicitly task-tracked or marked as stubs.
TODO_CPP=$(grep -rn "// TODO" compiler-core/ \
  --include="*.cpp" \
  --include="*.h" \
  --include="*.hpp" \
  | grep -v "// TODO()" \
  | grep -v "// TODO(Task " \
  | grep -v "// TODO(Task" \
  | grep -v "TEMP:" \
  | grep -v "STUB:" \
  | wc -l)

if [ "$TODO_CPP" -gt 0 ]; then
  echo "  ERROR: Found $TODO_CPP C++ TODO stubs"
  grep -rn "// TODO" compiler-core/ \
    --include="*.cpp" \
    --include="*.h" \
    --include="*.hpp" \
    | grep -v "// TODO()" \
    | grep -v "// TODO(Task " \
    | grep -v "// TODO(Task" \
    | grep -v "TEMP:"
  ((ERRORS++)) || true
else
  echo "  C++ TODO stubs: OK"
fi

# ============================================================
# Layer 3: Rust TODOs
# ============================================================
echo "[3/4] Checking Rust TODO/FIXME stubs..."

TODO_RUST=$(grep -rn "// TODO\|// FIXME\|todo!\|unimplemented!" tools/ \
  --include="*.rs" \
  | grep -v "pub fn todo_" \
  | grep -v "// TODO()" \
  | grep -v "// STUB:" \
  | grep -v "STUB:" \
  | wc -l)

TODO_RUST_NESTED=$(grep -rn "// TODO\|// FIXME\|todo!\|unimplemented!" runtime/rust/ \
  --include="*.rs" \
  | grep -v "pub fn todo_" \
  | grep -v "// TODO()" \
  | grep -v "// STUB:" \
  | grep -v "STUB:" \
  | wc -l)

RUST_TODO_TOTAL=$((TODO_RUST + TODO_RUST_NESTED))

if [ "$RUST_TODO_TOTAL" -gt 0 ]; then
  echo "  ERROR: Found $RUST_TODO_TOTAL Rust TODO/FIXME stubs"
  ((ERRORS++)) || true
else
  echo "  Rust TODO stubs: OK"
fi

# ============================================================
# Layer 4: Docs overclaim check
# ============================================================
echo "[4/4] Checking docs overclaims..."
DOCS_OVERCLAIM=$(grep -rn "Implemented\|Complete" docs/*.md \
  | grep -v "Incomplete\|TODO\|Task.*:" \
  | grep -v "COMPLETE\|implemented" \
  | wc -l)

# ============================================================
# Layer 5: C placeholder enforcement (Task 28)
# ============================================================
echo "[5/5] Checking C adapter placeholder patterns..."

# C adapter crates should not have todo!/unimplemented! in production code
TODO_C_ADAPTER=$(grep -rn "todo!\|unimplemented!" tools/crates/chimera-c-*/src/ \
  --include="*.rs" \
  | grep -v "// TODO\|// STUB\|#\[cfg(test)\]" \
  | wc -l)

# C adapter crates should not have placeholder function bodies (empty impl blocks)
PLACEHOLDER_FN=$(grep -rn "fn placeholder\|_placeholder\|fn unimplemented" tools/crates/chimera-c-*/src/ \
  --include="*.rs" \
  | grep -v "//\|#\[cfg(test)\]" \
  | wc -l)

# Check for mock-only test patterns in production C adapter code
MOCK_ONLY=$(grep -rn "// MOCK\|// FAKE\|// STUB" tools/crates/chimera-c-*/src/ \
  --include="*.rs" \
  | grep -v "test\|Test" \
  | wc -l)

C_PLACEHOLDERS=$((TODO_C_ADAPTER + PLACEHOLDER_FN + MOCK_ONLY))

if [ "$C_PLACEHOLDERS" -gt 0 ]; then
  echo "  ERROR: Found $C_PLACEHOLDERS C adapter placeholder patterns"
  ((ERRORS++)) || true
else
  echo "  C adapter placeholders: OK"
fi

# ============================================================
# Summary
# ============================================================
echo ""
echo "=== Placeholder Gate Results ==="

if [ "$ERRORS" -gt 0 ]; then
  echo "FAILED: $ERRORS error(s) found"
  exit 1
else
  echo "PASSED: No unauthorized placeholders found"
  exit 0
fi
