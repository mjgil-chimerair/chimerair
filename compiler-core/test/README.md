# Chimera Compiler Core Tests

## Test Structure

```
test/
├── Dialect/
│   └── Chimera/           # Chimera dialect lit tests
│       ├── driver.test    # Driver smoke test
│       └── parsing.test   # Parsing tests
└── lit.cfg.py.in          # Lit configuration template
```

## Running Tests

From build directory:

```bash
ctest --output-on-failure
```

Or run lit directly:

```bash
llvm-lit test/Dialect
```

## Adding New Tests

1. Add a `.test` file in the appropriate directory
2. Use `// RUN:` directives for FileCheck commands
3. Use `// CHECK:` directives for expected output

Example:

```mlir
// RUN: chimera-opt %s 2>&1 | FileCheck %s

// CHECK-LABEL: module
module() {
  // CHECK: func @hello
  func @hello() : () -> ()
}
```