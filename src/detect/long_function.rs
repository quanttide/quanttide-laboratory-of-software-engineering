use std::path::PathBuf;

use super::{Detector, Finding, Severity};

const MAY_THRESHOLD: usize = 30;
const SHOULD_THRESHOLD: usize = 50;
const MUST_THRESHOLD: usize = 80;

const FUNCTION_NODE_KINDS: &[&str] = &["function_item", "function_definition", "function_declaration", "method_declaration"];

pub struct LongFunctionDetector;

impl Detector for LongFunctionDetector {
    fn rule_id(&self) -> &'static str {
        "long-function"
    }

    fn description(&self) -> &'static str {
        "函数体过长"
    }

    fn detect(&self, source: &str, tree: &tree_sitter::Tree, file_path: &PathBuf) -> Vec<Finding> {
        let mut findings = Vec::new();
        super::walk_tree(tree, |node| {
            if FUNCTION_NODE_KINDS.contains(&node.kind()) {
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
        });
        findings
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
    if let Some(name) = find_identifier_in_children(node, source) {
        return name;
    }
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if let Some(name) = find_identifier_in_children(&child, source) {
                return name;
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    "<anonymous>".to_string()
}

fn find_identifier_in_children(node: &tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.kind() == "identifier" {
                if let Ok(s) = child.utf8_text(source.as_bytes()) {
                    return Some(s.to_string());
                }
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    None
}
