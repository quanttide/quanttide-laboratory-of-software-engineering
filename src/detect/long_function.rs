use std::path::PathBuf;

use super::{Detector, Finding, Severity};

pub struct LongFunctionDetector;

impl Detector for LongFunctionDetector {
    fn rule_id(&self) -> &'static str {
        "rust-long-function"
    }

    fn description(&self) -> &'static str {
        "函数体过长"
    }

    fn detect(&self, source: &str, tree: &tree_sitter::Tree, file_path: &PathBuf) -> Vec<Finding> {
        let mut findings = Vec::new();
        let mut cursor = tree.walk();
        let max_lines = 60;

        loop {
            let node = cursor.node();
            if node.kind() == "function_item" {
                let start = node.start_position().row;
                let end = node.end_position().row;
                let body_lines = end - start;

                if body_lines > max_lines {
                    let name = extract_function_name(&node, source);
                    findings.push(Finding {
                        file_path: file_path.clone(),
                        line: start + 1,
                        column: 1,
                        severity: Severity::Should,
                        rule_id: self.rule_id().to_string(),
                    message: format!(
                        "函数 `{}` 共 {} 行，建议不超过 60 行",
                        name, body_lines,
                    ),
                    });
                }
            }

            if cursor.goto_first_child() {
                continue;
            }
            loop {
                if cursor.goto_next_sibling() {
                    break;
                }
                if !cursor.goto_parent() {
                    return findings;
                }
            }
        }
    }
}

fn extract_function_name(node: &tree_sitter::Node, source: &str) -> String {
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.kind() == "identifier" {
                if let Ok(s) = child.utf8_text(source.as_bytes()) {
                    return s.to_string();
                }
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    "<anonymous>".to_string()
}
