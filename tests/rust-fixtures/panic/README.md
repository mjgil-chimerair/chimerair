# Panic Rust FFI Fixture

This fixture provides panic boundary testing functions for testing the Chimera Rust panic policy handling.

## Purpose

The panic fixture is used to verify panic boundary behavior:

1. Parse Rust source with `chimera-rust-source`
2. Extract effect facts with `chimera-rust-effects` (may_panic)
3. Lower to ChimeraIR with `chimera-rust-to-chimera`
4. Verify through `compiler-core` with panic boundary policies

## Panic Policy

| Policy | Behavior |
|--------|----------|
| `abort` | No unwinding, compile-time enforcement |
| `unwind` | Allowed across `extern "C-unwind"` boundaries |
| `catch_unwind` | Runtime catch via `catch_panic` helper |

## PanicStatus Type

| Field | Type | Description |
|-------|------|-------------|
| `did_not_panic` | `bool` | True if function did not panic |
| `panic_payload` | `u64` | Panic identifier (0 = success) |

## Exported Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `no_panic_function` | `fn(i32) -> i32` | Never panics (safe for FFI) |
| `may_panic_function` | `fn(i32) -> i32` | May panic (requires catch_unwind) |
| `catch_panic_wrapper` | `fn(unsafe extern "C" fn(i32) -> i32, i32) -> PanicStatus` | Catch panics from function call |
| `safe_call_no_panic` | `fn(i32) -> PanicStatus` | Call no_panic_function safely |
| `safe_call_may_panic` | `fn(i32) -> PanicStatus` | Call may_panic_function with catch |
| `self_catching_function` | `fn(i32) -> PanicStatus` | Self-contained panic catching |
| `panic_status_is_ok` | `fn(PanicStatus) -> bool` | Check if status is OK |
| `panic_status_payload` | `fn(PanicStatus) -> u64` | Get panic payload |

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```