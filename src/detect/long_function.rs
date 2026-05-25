use std::path::PathBuf;

use super::{Detector, Finding, Severity};

const MAY_THRESHOLD: usize = 30;
const SHOULD_THRESHOLD: usize = 50;
const MUST_THRESHOLD: usize = 80;

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

        loop {
            let node = cursor.node();
            if node.kind() == "function_item" {
                let start = node.start_position().row;
                let end = node.end_position().row;
                let body_lines = end - start;

                if let Some(severity) = classify(body_lines) {
                    let name = extract_function_name(&node, source);
                    findings.push(Finding {
                        file_path: file_path.clone(),
                        line: start + 1,
                        column: 1,
                        severity,
                        rule_id: self.rule_id().to_string(),
                        message: format!("函数 `{}` 共 {} 行", name, body_lines),
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

fn classify(lines: usize) -> Option<Severity> {
    if lines > MUST_THRESHOLD {
        Some(Severity::Must)
    } else if lines > SHOULD_THRESHOLD {
        Some(Severity::Should)
    } else if lines > MAY_THRESHOLD {
        Some(Severity::May)
    } else {
        None
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
