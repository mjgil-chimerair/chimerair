# Unsafe Rust FFI Fixture

This fixture provides raw pointer operations and unsafe extern testing functions for testing the Chimera Rust unsafe boundary handling.

## Purpose

The unsafe fixture is used to verify unsafe boundary behavior:

1. Parse Rust source with `chimera-rust-source`
2. Extract effect facts with `chimera-rust-effects` (unsafe)
3. Lower to ChimeraIR with `chimera-rust-to-chimera`
4. Verify through `compiler-core` with TCB trust ledger verification

## Trust Levels

| Level     | Value | Description |
|-----------|-------|-------------|
| Untrusted | 0     | No trust assumptions |
| Trusted   | 1     | Caller verified safety preconditions |
| TCB       | 2     | Trusted computing base - highest trust |

## TrustLedgerEntry Type

| Field          | Type    | Description |
|----------------|---------|-------------|
| operation_id   | u32     | Operation identifier |
| trust_level    | u8      | Trust level (0-2) |
| was_checked    | bool    | Whether entry was verified |
| line_number    | u32     | Source line of operation |

## Exported Functions

| Function                  | Signature                              | Description |
|---------------------------|----------------------------------------|-------------|
| `raw_u64_create`          | `fn(u64) -> *mut u64`                  | Create heap-allocated u64 |
| `raw_u64_deref`           | `fn(*mut u64) -> DerefResult`         | Dereference raw pointer |
| `raw_u64_new_and_deref`   | `fn(u64) -> DerefResult`               | Create and dereference |
| `raw_u64_write`           | `fn(*mut u64, u64) -> bool`            | Write to raw pointer |
| `raw_u64_swap`            | `fn(*mut u64, *mut u64) -> bool`       | Swap values at pointers |
| `tcb_verify_and_commit`   | `fn(*mut TrustLedgerEntry, u32) -> bool` | TCB verification |
| `record_unsafe_operation` | `fn(*mut TrustLedgerEntry, u32, TrustLevel, u32)` | Record operation |
| `verify_trust_entry`      | `fn(*mut TrustLedgerEntry) -> bool`     | Verify entry checked |
| `raw_pointer_add`        | `fn(*const u64, usize) -> u64`         | Pointer arithmetic |
| `pointer_is_aligned`     | `fn(*const u64, u64) -> bool`          | Check alignment |
| `unsafe_invariant_check` | `fn(*mut u64, usize, u64) -> bool`     | Invariant verification |
| `get_minimum_trust_level` | `fn() -> TrustLevel`                   | Query minimum trust |

## DerefResult Type

| Field       | Type   | Description |
|-------------|--------|-------------|
| value       | u64    | Dereferenced value |
| was_valid   | bool   | Whether pointer was valid |

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```