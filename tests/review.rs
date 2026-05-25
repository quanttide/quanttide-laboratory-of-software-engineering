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

#[test]
fn test_review_with_rules_long_parameter_list() {
    let fixture = fixture_path();
    let output = cli()
        .arg("review")
        .arg(&fixture)
        .arg("--rules")
        .arg("long-parameter-list")
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn test_review_with_multiple_rules() {
    let fixture = fixture_path();
    let output = cli()
        .arg("review")
        .arg(&fixture)
        .arg("--rules")
        .arg("long-function,unsafe-block")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn test_review_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    let output = cli()
        .arg("review")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("未发现问题"));
}

#[test]
fn test_review_status_flag() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    std::fs::create_dir(&src).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").unwrap();
    std::fs::write(src.join("lib.rs"), "pub fn f() -> i32 { 42 }\n").unwrap();
    let output = cli()
        .arg("review")
        .arg(dir.path())
        .arg("--status")
        .output()
        .unwrap();
    assert!(output.status.success());
    let status_path = dir.path().join("STATUS.md");
    assert!(status_path.exists());
    let content = std::fs::read_to_string(status_path).unwrap();
    assert!(content.contains("Code Scan Status"));
}

#[test]
fn test_review_unknown_format_defaults_to_terminal() {
    let fixture = fixture_path();
    let output = cli()
        .arg("review")
        .arg(&fixture)
        .arg("--format")
        .arg("unknown")
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn test_review_reflect_flag() {
    let fixture = fixture_path();
    let output = cli()
        .arg("review")
        .arg(&fixture)
        .arg("--reflect")
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn test_review_dep_graph_rule() {
    let fixture = fixture_path();
    let output = cli()
        .arg("review")
        .arg(&fixture)
        .arg("--rules")
        .arg("dep-graph")
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn test_review_reflect_with_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    let output = cli()
        .arg("review")
        .arg(dir.path())
        .arg("--reflect")
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn test_review_dead_code_and_depgraph_together() {
    let fixture = fixture_path();
    let output = cli()
        .arg("review")
        .arg(&fixture)
        .arg("--rules")
        .arg("dep-graph")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn test_review_status_without_cargo_project() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("sub");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("f.rs"), "pub fn f() {}\n").unwrap();
    // 没有 Cargo.toml → find_project_root 返回 None → STATUS.md 不会创建
    let output = cli()
        .arg("review")
        .arg(&sub)
        .arg("--status")
        .output()
        .unwrap();
    assert!(output.status.success());
    // 没有 STATUS.md 被创建
    assert!(!dir.path().join("STATUS.md").exists());
}

#[test]
fn test_review_with_invalid_rust_syntax() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("bad.rs"), "fn f(}").unwrap();
    let output = cli()
        .arg("review")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn test_review_status_with_cargo_in_parent_find_none() {
    // 测试在 Cargo 项目的子目录中运行 --status
    // find_project_root 从子目录往上能找到 Cargo.toml
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").unwrap();
    let sub = dir.path().join("src");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("lib.rs"), "pub fn f() {}\n").unwrap();
    let output = cli()
        .arg("review")
        .arg(&sub)
        .arg("--status")
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn test_review_with_all_project_rules() {
    let fixture = fixture_path();
    let output = cli()
        .arg("review")
        .arg(&fixture)
        .arg("--rules")
        .arg("missing-tests,dep-graph,unused-variable")
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn test_review_reflect_with_json() {
    let fixture = fixture_path();
    let output = cli()
        .arg("review")
        .arg(&fixture)
        .arg("--reflect")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.is_array());
}
