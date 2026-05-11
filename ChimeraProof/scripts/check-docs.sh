#!/bin/bash
# Check that required documentation files exist

set -e

echo "Checking documentation files..."
# Docs are in parent directory (repo root)
test -f ../docs/build.md || { echo "Missing docs/build.md"; exit 1; }
test -f ../docs/testing.md || { echo "Missing docs/testing.md"; exit 1; }
test -f ../docs/ci.md || { echo "Missing docs/ci.md"; exit 1; }
test -f ../docs/repo-layout.md || { echo "Missing docs/repo-layout.md"; exit 1; }
test -f ../docs/trusted-computing-base.md || { echo "Missing docs/trusted-computing-base.md"; exit 1; }
echo "Documentation check passed!"