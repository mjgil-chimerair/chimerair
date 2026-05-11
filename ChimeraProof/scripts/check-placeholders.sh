#!/bin/bash
# Check for unauthorized sorry/admit placeholders in production code

set -e

cd "$(dirname "$0")/.."

echo "Checking for sorry placeholders..."
SORRY_COUNT=$(grep -rn "sorry" Chimera/ \
  --include="*.lean" \
  --exclude-dir=.lake \
  | grep -v "PROOF" \
  | grep -v "OPEN" \
  | wc -l)

if [ "$SORRY_COUNT" -gt 0 ]; then
  echo "ERROR: Found $SORRY_COUNT unauthorized sorry placeholders"
  grep -rn "sorry" Chimera/ \
    --include="*.lean" \
    --exclude-dir=.lake \
    | grep -v "PROOF" \
    | grep -v "OPEN"
  exit 1
fi

echo "Checking for undocumented admits..."
ADMIT_COUNT=$(grep -rn "admit" Chimera/ \
  --include="*.lean" \
  --exclude-dir=.lake \
  | grep -v "PROOF" \
  | wc -l)

if [ "$ADMIT_COUNT" -gt 0 ]; then
  echo "WARNING: Found $ADMIT_COUNT admit statements (allowed if documented)"
fi

echo "Placeholder check passed!"