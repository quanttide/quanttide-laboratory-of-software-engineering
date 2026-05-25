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
        if !line.starts_with('{') {
            continue;
        }

        let Ok(msg) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };

        if msg["reason"] != "compiler-message" {
            continue;
        }
        if msg["message"]["level"] != "warning" {
            continue;
        }

        let code = match msg["message"]["code"]["code"].as_str() {
            Some(c) => c,
            None => continue,
        };
        if !WARNING_CODES.contains(&code) {
            continue;
        }

        let spans = match msg["message"]["spans"].as_array() {
            Some(s) => s,
            None => continue,
        };
        let Some(span) = spans.first() else { continue };

        let file_name = match span["file_name"].as_str() {
            Some(f) => f,
            None => continue,
        };
        let line = match span["line_start"].as_u64() {
            Some(l) => l as usize,
            None => continue,
        };
        let column = match span["column_start"].as_u64() {
            Some(c) => c as usize,
            None => 1,
        };

        let msg_text = match msg["message"]["message"].as_str() {
            Some(m) => m.to_string(),
            None => continue,
        };

        let file_path = project_root.join(file_name);

        findings.push(Finding {
            file_path,
            line,
            column,
            severity: Severity::Should,
            rule_id: RULE_ID.to_string(),
            message: msg_text,
        });
    }

    Ok(findings)
}
