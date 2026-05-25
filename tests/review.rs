use std::path::PathBuf;
use std::process::Command;

fn fixture_path() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().unwrap().join("default")
}

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_qtcloud-code"))
}

#[test]
fn test_review_fixture_dir_exists() {
    let path = fixture_path();
    assert!(path.exists(), "fixture 目录不存在: {}", path.display());
    assert!(path.join("Cargo.toml").exists());
}

#[test]
fn test_review_help_succeeds() {
    let output = cli().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("review"));
}

#[test]
fn test_review_default_repo() {
    let fixture = fixture_path();
    let output = cli()
        .arg("review")
        .arg(&fixture)
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn test_review_json_format() {
    let fixture = fixture_path();
    let output = cli()
        .arg("review")
        .arg(&fixture)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn test_review_with_rules_filter() {
    let fixture = fixture_path();
    let output = cli()
        .arg("review")
        .arg(&fixture)
        .arg("--rules")
        .arg("long-function")
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn test_review_invalid_path() {
    let output = cli()
        .arg("review")
        .arg("/nonexistent/path")
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn test_list_rules() {
    let output = cli().arg("list-rules").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("long-function"));
    assert!(stdout.contains("unused-variable"));
}
