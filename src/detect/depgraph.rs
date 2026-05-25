use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph_no_findings() {
        let g = DepGraph { nodes: vec![], edges: vec![] };
        assert!(check(&g).is_empty());
    }

    #[test]
    fn test_cycle_detection() {
        let g = DepGraph {
            nodes: vec!["a".into(), "b".into()],
            edges: vec![("a".into(), "b".into()), ("b".into(), "a".into())],
        };
        let f = check(&g);
        assert!(f.iter().any(|x| x.message.contains("循环依赖")));
    }

    #[test]
    fn test_orphan_detection() {
        let g = DepGraph {
            nodes: vec!["a".into(), "b".into()],
            edges: vec![("a".into(), "b".into())],
        };
        let f = check(&g);
        // a 依赖 b, a 不孤立; b 被 a 依赖, 不孤立
        assert!(!f.iter().any(|x| x.message.contains("孤立")));
    }

    #[test]
    fn test_high_fan_in() {
        let nodes: Vec<String> = (0..8).map(|i| format!("m{}", i)).collect();
        let edges: Vec<(String, String)> = (1..8).map(|i| (format!("m{}", i), "m0".into())).collect();
        let g = DepGraph { nodes, edges };
        let f = check(&g);
        assert!(f.iter().any(|x| x.message.contains("高扇入")));
    }

    #[test]
    fn test_high_fan_out() {
        let nodes: Vec<String> = (0..8).map(|i| format!("m{}", i)).collect();
        let edges: Vec<(String, String)> = (1..8).map(|i| ("m0".into(), format!("m{}", i))).collect();
        let g = DepGraph { nodes, edges };
        let f = check(&g);
        assert!(f.iter().any(|x| x.message.contains("高扇出")));
    }

    #[test]
    fn test_reverse_dep_slice() {
        let g = DepGraph {
            nodes: vec!["a".into(), "b".into(), "c".into()],
            edges: vec![("a".into(), "c".into()), ("b".into(), "c".into())],
        };
        let r = reverse_dep_slice(&g, "c");
        assert_eq!(r.len(), 2);
        assert!(r.contains(&"a".to_string()));
        assert!(r.contains(&"b".to_string()));
    }
}
use std::path::{Path, PathBuf};
use std::fs;

use crate::detect::{Finding, Severity};

pub const RULE_ID: &str = "dep-graph";
pub const DESCRIPTION: &str = "模块依赖图异常";

/// 模块依赖图
#[derive(Debug, Clone)]
pub struct DepGraph {
    pub nodes: Vec<String>,          // 所有模块名
    pub edges: Vec<(String, String)>, // (from, to) 依赖关系
}

/// 构建项目的模块依赖图
pub fn build_dep_graph(root: &Path) -> DepGraph {
    let mut nodes_set = HashSet::new();
    let mut edges = Vec::new();

    let src_dir = root.join("src");
    if !src_dir.exists() { return DepGraph { nodes: vec![], edges: vec![] }; }

    // 收集所有 .rs 文件
    let files = collect_rs_files(&src_dir);

    for file_path in &files {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let module_path = file_to_module_path(file_path, root);
        nodes_set.insert(module_path.clone());

        for line in content.lines() {
            let line = line.trim();
            // mod xxx; (子模块)
            if line.starts_with("mod ") && line.ends_with(';') {
                let name = line[4..line.len() - 1].trim().to_string();
                if !name.contains('{') && !name.contains(' ') {
                    let sub_module = if module_path.is_empty() {
                        name.clone()
                    } else {
                        format!("{}::{}", module_path, name)
                    };
                    edges.push((module_path.clone(), sub_module));
                }
            }
            // use crate::xxx::yyy;
            if line.starts_with("use ") && line.ends_with(';') {
                let use_path = line[4..line.len() - 1].trim();
                if use_path.starts_with("crate::") {
                    let target = use_path[7..].to_string();
                    let target_module = target.split("::").next().unwrap_or("").to_string();
                    if !target_module.is_empty() {
                        edges.push((module_path.clone(), target_module));
                    }
                }
            }
            // pub use crate::xxx;
            if line.starts_with("pub use ") && line.ends_with(';') {
                let use_path = line[8..line.len() - 1].trim();
                if use_path.starts_with("crate::") {
                    let target = use_path[7..].to_string();
                    let target_module = target.split("::").next().unwrap_or("").to_string();
                    if !target_module.is_empty() {
                        edges.push((module_path.clone(), target_module));
                    }
                }
            }
        }
    }

    // 确保所有目标节点存在
    for (_, to) in &edges {
        nodes_set.insert(to.clone());
    }

    let mut nodes: Vec<String> = nodes_set.into_iter().collect();
    nodes.sort();

    DepGraph { nodes, edges }
}

/// 反向依赖切片：找出直接依赖某模块的所有模块
pub fn reverse_dep_slice(graph: &DepGraph, target: &str) -> Vec<String> {
    let mut result: Vec<String> = graph.edges.iter()
        .filter(|(_, to)| to == target || to.starts_with(&format!("{}::", target)))
        .map(|(from, _)| from.clone())
        .collect();
    result.sort();
    result.dedup();
    result
}

/// 正向依赖切片：从某模块出发找出所有直接依赖它的模块
pub fn forward_dep_slice(graph: &DepGraph, source: &str) -> Vec<String> {
    let mut result: Vec<String> = graph.edges.iter()
        .filter(|(from, _)| from == source)
        .map(|(_, to)| to.clone())
        .collect();
    result.sort();
    result.dedup();
    result
}

/// 获取完整的依赖链（BFS 反向）
pub fn reverse_dep_chain(graph: &DepGraph, target: &str) -> Vec<Vec<String>> {
    let mut chains = Vec::new();
    let direct = reverse_dep_slice(graph, target);
    for dep in &direct {
        chains.push(vec![dep.clone(), target.to_string()]);
        // 递归一层
        for indirect in reverse_dep_slice(graph, dep) {
            chains.push(vec![indirect, dep.clone(), target.to_string()]);
        }
    }
    chains
}

// ===== 内部工具 =====

fn collect_rs_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.file_name().and_then(|s| s.to_str()) != Some("bin") {
                files.extend(collect_rs_files(&path));
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
    files
}

fn file_to_module_path(file: &Path, root: &Path) -> String {
    let relative = file.strip_prefix(root).unwrap_or(file);
    let without_src = relative.strip_prefix("src").unwrap_or(relative);
    let path_str = without_src.to_string_lossy();

    // 移除 .rs 后缀
    let without_ext = path_str.trim_end_matches(".rs");
    // 替换路径分隔符为 ::
    let module = without_ext.replace('/', "::").replace('\\', "::");

    // mod.rs → 父模块名
    if module.ends_with("::mod") {
        let trimmed = module.trim_end_matches("::mod");
        if !trimmed.is_empty() { return trimmed.to_string(); }
    }

    if module.is_empty() || module == "main" || module == "lib" {
        return String::new(); // crate 根
    }

    module.to_string()
}

pub fn check(graph: &DepGraph) -> Vec<Finding> {
    let mut findings = Vec::new();
    let file = PathBuf::from(".");

    // 1. 循环依赖检测
    for (from, to) in &graph.edges {
        if graph.edges.iter().any(|(f, t)| f == to && t == from) {
            findings.push(Finding {
                file_path: file.clone(),
                line: 1, column: 1,
                severity: Severity::Must,
                rule_id: RULE_ID.to_string(),
                message: format!("循环依赖: {} ↔ {}", from, to),
            });
        }
    }

    // 2. 高扇入（太多模块依赖它）
    for node in &graph.nodes {
        let fan_in = graph.edges.iter().filter(|(_, to)| to == node).count();
        if fan_in > 5 {
            findings.push(Finding {
                file_path: file.clone(),
                line: 1, column: 1,
                severity: Severity::Should,
                rule_id: RULE_ID.to_string(),
                message: format!("高扇入: {} 被 {} 个模块依赖", node, fan_in),
            });
        }
    }

    // 3. 高扇出（依赖太多模块）
    for node in &graph.nodes {
        if node.is_empty() { continue; }
        let fan_out = graph.edges.iter().filter(|(from, _)| from == node).count();
        if fan_out > 5 {
            findings.push(Finding {
                file_path: file.clone(),
                line: 1, column: 1,
                severity: Severity::Should,
                rule_id: RULE_ID.to_string(),
                message: format!("高扇出: {} 依赖 {} 个模块", node, fan_out),
            });
        }
    }

    // 4. 孤立模块（无模块依赖它，也非根模块）
    for node in &graph.nodes {
        if node.is_empty() { continue; }
        let fan_in = graph.edges.iter().filter(|(_, to)| to == node).count();
        let fan_out = graph.edges.iter().filter(|(from, _)| from == node).count();
        if fan_in == 0 && fan_out == 0 {
            findings.push(Finding {
                file_path: file.clone(),
                line: 1, column: 1,
                severity: Severity::May,
                rule_id: RULE_ID.to_string(),
                message: format!("孤立模块: {} 不被任何模块依赖", node),
            });
        }
    }

    findings
}
