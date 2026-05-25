use std::collections::HashSet;
use std::path::Path;
use std::fs;

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
