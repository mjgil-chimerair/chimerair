# C Fixtures

This directory contains C fixture tests for the Chimera C adapter.

## Fixtures

- **basic** - Basic smoke test with simple functions and a struct (Task 146)
- **header-only** - Header with structs, typedefs, function declarations (Task 147)
- **layout** - Struct layout tests with packed/aligned fields (Task 149)
- **bitfields** - Bitfield struct tests (Task 150)
- **errors** - errno/status error handling tests (Task 152)
- **callbacks** - Function pointer callback tests (Task 153)

## Running Tests

Compile and test all fixtures:

```bash
# Compile all fixtures
cd tests/c-fixtures
for dir in */; do
  if [ -f "$dir/compile_commands.json" ]; then
    cd "$dir"
    gcc -c -I. *.c 2>/dev/null || true
    cd ..
  fi
done

# Verify with Clang
clang -fsyntax-only -Ibasic basic.h 2>/dev/null || true
```

## Fixture Requirements

Each fixture must:
1. Compile with standard C compilers (gcc, clang)
2. Be consumed by chimera-c-clang extraction
3. Pass layout verification tests
4. Include compile_commands.json where applicable

## Adding New Fixtures

1. Create a new directory under `tests/c-fixtures/`
2. Add header/source files with descriptive names
3. Include `compile_commands.json` for compile database tests
4. Add tests in the appropriate Rust crate
5. Update this README with the new fixture