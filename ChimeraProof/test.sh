#!/bin/bash
# Test driver: run lake build as a test
set -e
cd "$(dirname "$0")"
echo "Running ChimeraProof build test..."
lake build
