#!/bin/bash
# Check documentation links and paths across the repo
# CI gate to fail on broken links, stale paths, or nonexistent files

set -e

cd "$(dirname "$0")/.."

ERRORS=0

echo "=== Chimera Docs Link/Path Checker ==="
echo ""

# ============================================================
# Layer 1: Required docs existence
# ============================================================
echo "[1/4] Checking required documentation files..."

REQUIRED_DOCS=(
    "docs/spec.md"
    "docs/build.md"
    "docs/testing.md"
    "docs/ci.md"
    "docs/repo-layout.md"
    "docs/task-list-7.md"
    "docs/trusted-computing-base.md"
    "docs/abi.md"
    "docs/diagnostics.md"
    "docs/versioning.md"
    "docs/artifact-flow.md"
    "docs/release-package-layout.md"
    "docs/passes.md"
    "docs/proof-bridge-format.md"
    "docs/project-manifest.md"
    "docs/release-checklist.md"
)

MISSING=0
for doc in "${REQUIRED_DOCS[@]}"; do
    if [ ! -f "$doc" ]; then
        echo "  ERROR: Missing required doc: $doc"
        ((MISSING++)) || true
    fi
done

if [ "$MISSING" -gt 0 ]; then
    echo "  ERROR: $MISSING required docs missing"
    ((ERRORS++)) || true
else
    echo "  Required docs: OK (${#REQUIRED_DOCS[@]} files)"
fi

# ============================================================
# Layer 2: Check for stale task list references
# ============================================================
echo "[2/4] Checking task list references..."

# Report active task lists at the repo root. Multiple active lists are allowed
# when they cover distinct execution tracks.
ACTIVE_TASK_LISTS=$(find docs -maxdepth 1 -name "task-list-*.md" -type f | wc -l)
if [ "$ACTIVE_TASK_LISTS" -gt 0 ]; then
    echo "  Active task lists:"
    find docs -maxdepth 1 -name "task-list-*.md" -type f | sort
else
    echo "  ERROR: No active task list found in docs/"
    ((ERRORS++)) || true
fi

# Check archived docs exist in archive
if [ -d "docs/archive" ]; then
    ARCHIVED_COUNT=$(find docs/archive -name "task-list-*.md" | wc -l)
    echo "  Archived task lists: $ARCHIVED_COUNT files in docs/archive/"
fi

# ============================================================
# Layer 3: Check for outdated doc references in CI
# ============================================================
echo "[3/4] Checking CI doc references..."

CI_OLD_REFS=0
if grep -q "task-list-6-code.md" .github/workflows/ci.yml 2>/dev/null; then
    echo "  ERROR: CI still references old task-list-6-code.md"
    ((CI_OLD_REFS++)) || true
fi

if grep -q "task-list-5-code.md" .github/workflows/ci.yml 2>/dev/null; then
    echo "  ERROR: CI still references old task-list-5-code.md"
    ((CI_OLD_REFS++)) || true
fi

if grep -q "task-list-5-lean.md" .github/workflows/ci.yml 2>/dev/null; then
    echo "  ERROR: CI still references old task-list-5-lean.md"
    ((CI_OLD_REFS++)) || true
fi

if [ "$CI_OLD_REFS" -eq 0 ]; then
    echo "  CI doc refs: OK"
else
    ((ERRORS++)) || true
fi

# ============================================================
# Layer 4: Check for path consistency in docs
# ============================================================
echo "[4/4] Checking doc path consistency..."

# Check that docs reference each other with correct relative paths
DOC_REF_ERRORS=0

# Example: spec.md should reference task-list-7.md not task-list-6.md
if grep -q "task-list-6" docs/spec.md 2>/dev/null; then
    echo "  WARN: docs/spec.md references old task-list-6"
    ((DOC_REF_ERRORS++)) || true
fi

if [ "$DOC_REF_ERRORS" -eq 0 ]; then
    echo "  Doc path consistency: OK"
fi

# ============================================================
# Layer 5: Final design doc link validation
# ============================================================
echo "[5/5] Checking ChimeraIR Final Design doc links..."

FINAL_DESIGN_DOC="docs/chimerair-final-design.md"
if [ ! -f "$FINAL_DESIGN_DOC" ]; then
    echo "  ERROR: Final design doc missing: $FINAL_DESIGN_DOC"
    ((ERRORS++)) || true
else
    FINAL_DESIGN_MISSING=0
    FINAL_DESIGN_REQUIRED_FROM=(
        "docs/architecture.md"
        "docs/artifact-flow.md"
        "docs/project-manifest.md"
        "docs/release-checklist.md"
    )
    for required_doc in "${FINAL_DESIGN_REQUIRED_FROM[@]}"; do
        if [ ! -f "$required_doc" ]; then
            echo "  ERROR: Required doc missing: $required_doc"
            ((FINAL_DESIGN_MISSING++)) || true
        elif ! grep -q "chimerair-final-design" "$required_doc" 2>/dev/null; then
            echo "  ERROR: $required_doc does not link to $FINAL_DESIGN_DOC"
            ((FINAL_DESIGN_MISSING++)) || true
        fi
    done

    if [ "$FINAL_DESIGN_MISSING" -gt 0 ]; then
        echo "  ERROR: $FINAL_DESIGN_MISSING required doc(s) missing or missing links to final design"
        ((ERRORS++)) || true
    else
        echo "  Final design doc links: OK (linked from all required docs)"
    fi

    # Verify supersession rules
    if grep -q "Supersession Rules" "$FINAL_DESIGN_DOC" 2>/dev/null; then
        echo "  Supersession rules: OK (present in final design doc)"
    else
        echo "  WARN: Supersession rules section missing from final design doc"
    fi
fi

# ============================================================
# Summary
# ============================================================
echo ""
echo "=== Docs Link/Path Check Results ==="

if [ "$ERRORS" -gt 0 ]; then
    echo "FAILED: $ERRORS error(s) found"
    exit 1
else
    echo "PASSED: All documentation checks passed"
    exit 0
fi
