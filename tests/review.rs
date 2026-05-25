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
    // 当前 review 仅为骨架，后续实装后验证检测结果
    assert!(output.status.success());
}
