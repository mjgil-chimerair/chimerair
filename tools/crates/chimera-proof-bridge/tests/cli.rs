use assert_cmd::Command;
use std::fs;

#[test]
fn test_cli_verify_accepts_valid_sidecar() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let sidecar = temp.path().join("valid.chproof");
    fs::write(
        &sidecar,
        r#"{
  "build_id": "example-module",
  "timestamp": 1,
  "target_triple": "x86_64-unknown-linux-gnu",
  "target_ptr_width": 64,
  "target_endian": "little",
  "obligations": [
    {
      "id": "layout_example",
      "kind": "layout",
      "target": "example_fn",
      "description": "layout check",
      "assumptions": []
    }
  ],
  "trust_assumptions": []
}"#,
    )
    .expect("write sidecar");

    Command::cargo_bin("chimera-proof-bridge")
        .expect("binary should build")
        .arg("verify")
        .arg(&sidecar)
        .assert()
        .success();
}

#[test]
fn test_cli_verify_rejects_invalid_sidecar() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let sidecar = temp.path().join("invalid.chproof");
    fs::write(
        &sidecar,
        r#"{
  "build_id": "",
  "timestamp": 1,
  "target_triple": "x86_64-unknown-linux-gnu",
  "target_ptr_width": 64,
  "target_endian": "little",
  "obligations": [],
  "trust_assumptions": []
}"#,
    )
    .expect("write sidecar");

    Command::cargo_bin("chimera-proof-bridge")
        .expect("binary should build")
        .arg("verify")
        .arg(&sidecar)
        .assert()
        .failure();
}

#[test]
fn test_cli_extract_from_artifacts_dir() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    fs::write(temp.path().join("snap.zsnap"), b"snapshot data").expect("write");
    fs::write(temp.path().join("dep.zdep"), b"dep graph").expect("write");
    fs::write(temp.path().join("meta.chmeta"), b"metadata").expect("write");

    Command::cargo_bin("chimera-proof-bridge")
        .expect("binary should build")
        .arg("extract")
        .arg(temp.path())
        .arg("test-component")
        .assert()
        .success();
}

#[test]
fn test_cli_extract_from_empty_dir_fails() {
    let temp = tempfile::TempDir::new().expect("temp dir");

    Command::cargo_bin("chimera-proof-bridge")
        .expect("binary should build")
        .arg("extract")
        .arg(temp.path())
        .arg("test-component")
        .assert()
        .failure();
}

#[test]
fn test_cli_extract_from_nonexistent_dir_fails() {
    Command::cargo_bin("chimera-proof-bridge")
        .expect("binary should build")
        .arg("extract")
        .arg("/nonexistent/path")
        .arg("test-component")
        .assert()
        .failure();
}

#[test]
fn test_cli_extract_with_proof_files() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    fs::write(temp.path().join("snap.zsnap"), b"snapshot").expect("write");
    fs::write(temp.path().join("lib.o"), b"object").expect("write");
    fs::write(temp.path().join("proof.chproof"), b"proof data").expect("write");

    Command::cargo_bin("chimera-proof-bridge")
        .expect("binary should build")
        .arg("extract")
        .arg(temp.path())
        .arg("multi-artifact")
        .assert()
        .success();
}

#[test]
fn test_cli_usage() {
    Command::cargo_bin("chimera-proof-bridge")
        .expect("binary should build")
        .assert()
        .failure()
        .stderr(predicates::str::contains("usage:"));
}

#[test]
fn test_cli_unknown_command() {
    Command::cargo_bin("chimera-proof-bridge")
        .expect("binary should build")
        .arg("unknown")
        .assert()
        .failure()
        .stderr(predicates::str::contains("usage:"));
}
