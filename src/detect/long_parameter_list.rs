use std::path::PathBuf;

use super::{Detector, Finding, Severity};

const MAY_THRESHOLD: usize = 4;
const SHOULD_THRESHOLD: usize = 6;
const MUST_THRESHOLD: usize = 9;

const FUNCTION_NODE_KINDS: &[&str] = &["function_item", "function_definition", "function_declaration", "method_declaration"];

pub struct LongParameterListDetector;

impl Detector for LongParameterListDetector {
    fn rule_id(&self) -> &'static str {
        "long-parameter-list"
    }

    fn description(&self) -> &'static str {
        "参数列表过长"
    }

    fn detect(&self, source: &str, tree: &tree_sitter::Tree, file_path: &PathBuf) -> Vec<Finding> {
        let mut findings = Vec::new();
        super::walk_tree(tree, |node| {
            if FUNCTION_NODE_KINDS.contains(&node.kind()) {
                let param_count = find_parameters_node(&node)
                    .as_ref().map(|p| count_params(p))
                    .unwrap_or(0);

                if let Some(severity) = classify(param_count) {
                    let name = extract_function_name(&node, source);
                    findings.push(Finding {
                        file_path: file_path.clone(),
                        line: node.start_position().row + 1,
                        column: 1,
                        severity,
                        rule_id: self.rule_id().to_string(),
                        message: format!("函数 `{}` 有 {} 个参数", name, param_count),
                    });
                }
            }
        });
        findings
    }
}

fn find_parameters_node<'tree>(node: &tree_sitter::Node<'tree>) -> Option<tree_sitter::Node<'tree>> {
    if let Some(params) = node.child_by_field_name("parameters") {
        return Some(params);
    }
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.kind() == "function_signature" {
                if let Some(params) = child.child_by_field_name("parameters") {
                    return Some(params);
                }
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    None
}

fn count_params(params_node: &tree_sitter::Node) -> usize {
    let mut cursor = params_node.walk();
    let mut has_go_style = false;
    let mut count = 0;
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.kind() == "parameter_declaration" {
                has_go_style = true;
                count += count_identifiers_in_node(&child);
            } else if child.is_named() {
                count += 1;
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    if !has_go_style {
        count = params_node.named_child_count();
    }
    count
}

fn count_identifiers_in_node(node: &tree_sitter::Node) -> usize {
    let mut count = 0;
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.kind() == "identifier" {
                count += 1;
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    count
}

fn classify(count: usize) -> Option<Severity> {
    if count > MUST_THRESHOLD {
        Some(Severity::Must)
    } else if count > SHOULD_THRESHOLD {
        Some(Severity::Should)
    } else if count > MAY_THRESHOLD {
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
