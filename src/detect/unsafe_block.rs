use std::path::PathBuf;

use super::{Detector, Finding, Severity};

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
        let mut cursor = tree.walk();
        let max_depth = 200;
        let mut depth = 0;

        loop {
            let node = cursor.node();
            if node.kind() == "unsafe_block" {
                let stmt_count = count_block_statements(&node);
                if stmt_count > 5 {
                    let pos = node.start_position();
                    findings.push(Finding {
                        file_path: file_path.clone(),
                        line: pos.row + 1,
                        column: pos.column + 1,
                        severity: Severity::Warning,
                        rule_id: self.rule_id().to_string(),
                        message: format!("unsafe 块包含 {} 条语句，建议控制在 5 条以内", stmt_count),
                    });
                }
            }

            if cursor.goto_first_child() && depth < max_depth {
                depth += 1;
                continue;
            }
            loop {
                if cursor.goto_next_sibling() {
                    break;
                }
                if !cursor.goto_parent() {
                    return findings;
                }
                depth -= 1;
            }
        }
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
