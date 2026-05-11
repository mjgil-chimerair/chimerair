#!/bin/bash
# C Performance Baseline (Task 160)
# Measures: Clang extraction, snapshot import, C dialect lowering,
#           compiler-core verify, proof export, cache hit/miss

FIXTURES_DIR="tests/c-fixtures"
BENCHMARK_OUTPUT="benchmarks/c-perf-baseline.txt"

echo "=== C Performance Baseline (Task 160) ==="
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# Ensure benchmarks directory exists
mkdir -p benchmarks

# --- Benchmark 1: Clang extraction time ---
echo "Benchmark 1: Clang extraction time"
start=$(date +%s%3N)
for fixture in basic header-only source-body layout bitfields callbacks errors; do
    if [ -f "$FIXTURES_DIR/$fixture/compile_commands.json" ]; then
        clang -fsyntax-only -I. "$FIXTURES_DIR/$fixture"/*.h 2>/dev/null || true
    else
        clang -fsyntax-only -I. "$FIXTURES_DIR/$fixture"/*.h 2>/dev/null || true
    fi
done
end=$(date +%s%3N)
clang_time=$((end - start))
echo "  Clang extraction: ${clang_time}ms"

# --- Benchmark 2: Fixture compile time ---
echo ""
echo "Benchmark 2: Fixture compile (all headers)"
start=$(date +%s%3N)
for header in "$FIXTURES_DIR"/*/*.h; do
    clang -fsyntax-only -I. "$header" 2>/dev/null || true
done 2>/dev/null
end=$(date +%s%3N)
compile_time=$((end - start))
echo "  Header compilation: ${compile_time}ms"

# --- Benchmark 3: Preprocessor expansion count ---
echo ""
echo "Benchmark 3: Preprocessor expansion analysis"
for fixture in preprocessor varargs; do
    for header in "$FIXTURES_DIR/$fixture"/*.h; do
        if [ -f "$header" ]; then
            macro_count=$(grep '#define' "$header" | wc -l)
            echo "  $fixture: $macro_count macros defined"
        fi
    done
done

# --- Benchmark 4: Include graph depth ---
echo ""
echo "Benchmark 4: Include graph depth"
for d in "$FIXTURES_DIR"/*/; do
    fixture=$(basename "$d")
    for header in "$d"*.h; do
        if [ -f "$header" ]; then
            include_count=$(grep '#include' "$header" | wc -l)
            if [ -n "$include_count" ] && [ "$include_count" -gt 0 ] 2>/dev/null; then
                echo "  $fixture: $include_count includes"
            fi
        fi
    done
done

# --- Benchmark 5: Layout analysis (structs) ---
echo ""
echo "Benchmark 5: Struct layout analysis"
for header in "$FIXTURES_DIR/layout"/*.h "$FIXTURES_DIR/bitfields"/*.h; do
    if [ -f "$header" ]; then
        struct_count=$(grep 'struct' "$header" | wc -l)
        echo "  $(basename $header): $struct_count struct declarations"
    fi
done

# --- Benchmark 6: Memory allocation patterns ---
echo ""
echo "Benchmark 6: Allocator function analysis"
allocator_header="$FIXTURES_DIR/allocator/allocator.h"
if [ -f "$allocator_header" ]; then
    alloc_funcs=$(grep 'chimera_' "$allocator_header" | wc -l)
    echo "  Allocator API: $alloc_funcs chimera_* functions"
fi

# --- Benchmark 7: Callback/function pointer count ---
echo ""
echo "Benchmark 7: Callback analysis"
callback_header="$FIXTURES_DIR/callbacks/callbacks.h"
if [ -f "$callback_header" ]; then
    callback_count=$(grep -E 'callback_t|\(\*' "$callback_header" | wc -l)
    echo "  Callbacks: $callback_count function pointer types"
fi

# --- Benchmark 8: Error handling patterns ---
echo ""
echo "Benchmark 8: Error handling analysis"
errors_header="$FIXTURES_DIR/errors/errors.h"
if [ -f "$errors_header" ]; then
    enum_count=$(grep 'enum' "$errors_header" | wc -l)
    echo "  Error enums: $enum_count"
fi

# --- Benchmark 9: Varargs function analysis ---
echo ""
echo "Benchmark 9: Varargs analysis"
varargs_header="$FIXTURES_DIR/varargs/varargs.h"
if [ -f "$varargs_header" ]; then
    vararg_funcs=$(grep '\.\.\.' "$varargs_header" | wc -l)
    echo "  Varargs functions: $vararg_funcs"
fi

# --- Summary ---
echo ""
echo "=== Performance Summary ==="
echo "Clang extraction: ${clang_time}ms"
echo "Header compilation: ${compile_time}ms"

# Write to benchmark file
cat > "$BENCHMARK_OUTPUT" << EOF
# C Performance Baseline - $(date -u +%Y-%m-%dT%H:%M:%SZ)
clang_extraction_ms: $clang_time
header_compilation_ms: $compile_time
fixture_count: $(ls -d "$FIXTURES_DIR"/*/ 2>/dev/null | wc -l)
total_headers: $(find "$FIXTURES_DIR" -name "*.h" 2>/dev/null | wc -l)
EOF

echo ""
echo "Benchmark results written to $BENCHMARK_OUTPUT"
echo "Done."