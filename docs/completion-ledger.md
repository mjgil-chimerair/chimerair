[task.7]
status = "Complete"
evidence = {
code = [
  "scripts/check-docs-links.sh",
  "scripts/release-gate.sh",
  "examples/one-binary/c-reader/chimera_reader.c"
]
tests = [
  "tests/completion-ledger.sh",
  "tests/release-gate-clean-checkout.sh"
]
docs = [
  "docs/task-list-7.md",
  "docs/chimerair-final-design.md",
  "docs/release-checklist.md"
]
ci_jobs = [
  "docs-exist",
  "docs-link-check",
  "release-gate-clean-checkout"
]
}
