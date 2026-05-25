use std::path::Path;
use std::process::Command;

use super::{Finding, Severity};

pub const RULE_ID: &str = "unused-variable";
pub const DESCRIPTION: &str = "未使用变量（通过 rustc/cargo check 检测）";

const WARNING_CODES: &[&str] = &["unused_variables", "unused_mut"];

pub fn check_compiler(project_root: &Path, enabled_rules: &[String]) -> Result<Vec<Finding>, String> {
    if !enabled_rules.contains(&RULE_ID.to_string()) {
        return Ok(vec![]);
    }

    let output = Command::new("cargo")
        .args(["check", "--message-format=json", "--quiet"])
        .current_dir(project_root)
        .output()
        .map_err(|e| format!("无法执行 cargo check: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut findings = Vec::new();

    for line in stdout.lines() {
        if let Some(finding) = parse_compiler_message(line, project_root) {
            findings.push(finding);
        }
    }

    Ok(findings)
}

fn parse_compiler_message(line: &str, project_root: &Path) -> Option<Finding> {
    if !line.starts_with('{') {
        return None;
    }

    let msg = serde_json::from_str::<serde_json::Value>(line).ok()?;

    if msg["reason"] != "compiler-message" || msg["message"]["level"] != "warning" {
        return None;
    }

    let code = msg["message"]["code"]["code"].as_str()?;
    if !WARNING_CODES.contains(&code) {
        return None;
    }

    let span = msg["message"]["spans"].as_array()?.first()?;
    let file_name = span["file_name"].as_str()?;
    let line = span["line_start"].as_u64()? as usize;
    let column = span["column_start"].as_u64().unwrap_or(1) as usize;
    let msg_text = msg["message"]["message"].as_str()?.to_string();

    Some(Finding {
        file_path: project_root.join(file_name),
        line,
        column,
        severity: Severity::Should,
        rule_id: RULE_ID.to_string(),
        message: msg_text,
    })
}
