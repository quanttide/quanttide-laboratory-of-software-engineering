use std::path::PathBuf;

use super::{Detector, Finding, Severity};

const MAY_THRESHOLD: usize = 3;
const SHOULD_THRESHOLD: usize = 5;
const MUST_THRESHOLD: usize = 8;

pub struct UnsafeBlockDetector;

impl Detector for UnsafeBlockDetector {
    fn rule_id(&self) -> &'static str {
        "rust-wide-unsafe"
    }

    fn description(&self) -> &'static str {
        "unsafe 块包含过多语句"
    }

    fn detect(&self, _source: &str, tree: &tree_sitter::Tree, file_path: &PathBuf) -> Vec<Finding> {
        let mut findings = Vec::new();
        super::walk_tree(tree, |node| {
            if node.kind() == "unsafe_block" {
                let stmt_count = count_block_statements(&node);
                if let Some(severity) = classify(stmt_count) {
                    let pos = node.start_position();
                    findings.push(Finding {
                        file_path: file_path.clone(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        severity,
                        rule_id: self.rule_id().to_string(),
                        message: format!("unsafe 块包含 {} 条语句", stmt_count),
                    });
                }
            }
        });
        findings
    }
}

fn count_block_statements(node: &tree_sitter::Node) -> usize {
    let mut count = 0;
    let mut cursor = node.walk();
    if !cursor.goto_first_child() {
        return 0;
    }
    loop {
        let child = cursor.node();
        if child.kind().ends_with("_statement") || child.kind() == "expression_statement" {
            count += 1;
        }
        if !cursor.goto_next_sibling() {
            break;
        }
    }
    count
}

fn classify(stmts: usize) -> Option<Severity> {
    if stmts > MUST_THRESHOLD {
        Some(Severity::Must)
    } else if stmts > SHOULD_THRESHOLD {
        Some(Severity::Should)
    } else if stmts > MAY_THRESHOLD {
        Some(Severity::May)
    } else {
        None
    }
}
