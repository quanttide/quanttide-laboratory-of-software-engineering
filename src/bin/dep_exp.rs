/// 模块依赖图实验：扫描源码中的 mod/use/pub 声明构建依赖图

use std::collections::{HashMap, HashSet};
use std::path::Path;

fn main() {
    let files = vec![
        ("src/main.rs", r#"mod config;
mod parser;
mod detector;
fn main() {}"#),
        ("src/config.rs", r#"use crate::detector::Finding;
pub fn load() {}"#),
        ("src/parser/mod.rs", r#"pub mod rust;
pub mod python;
use crate::config;"#),
        ("src/parser/rust.rs", r#"use crate::parser;
use crate::detector;"#),
        ("src/detector/mod.rs", r#"pub mod long_function;
pub use crate::config::CodeConfig;"#),
        ("src/detector/long_function.rs", r#"use crate::detector;
use crate::config;"#),
    ];

    let mut deps: HashMap<String, Vec<String>> = HashMap::new();

    for (path_str, source) in &files {
        let module_path = file_to_module(path_str);
        deps.entry(module_path.clone()).or_default();

        // 父模块对子模块的依赖：src/parser/mod.rs → parser::rust
        let (imports, submodules) = extract_deps(source);
        deps.get_mut(&module_path).unwrap().extend(imports);
        for sub in &submodules {
            let sub_path = format!("{}::{}", module_path, sub);
            deps.get_mut(&module_path).unwrap().push(sub_path.clone());
            deps.entry(sub_path).or_default();
        }
    }

    println!("=== 模块依赖图 ===");
    let mut sorted: Vec<_> = deps.keys().collect();
    sorted.sort();
    for module in &sorted {
        let deps_list = &deps[*module];
        println!("  {}", module);
        for d in deps_list {
            println!("    → {}", d);
        }
    }

    // 反向依赖切片
    for target in &["config", "detector"] {
        println!("\n=== 反向依赖切片：依赖 '{}' 的模块 ===", target);
        for (module, deps_list) in &deps {
            let deps_short: Vec<&str> = deps_list.iter().map(|d| {
                let parts: Vec<&str> = d.split("::").collect();
                parts[0]
            }).collect();
            if deps_short.iter().any(|d| *d == *target) {
                println!("  {} → {}", module, target);
            }
        }
    }

    // 正向依赖切片：从 main 出发
    println!("\n=== 正向依赖切片：从 'main' 出发 ===");
    let mut visited = HashSet::new();
    let mut queue = vec!["main".to_string()];
    while let Some(current) = queue.pop() {
        if !visited.insert(current.clone()) { continue; }
        if let Some(deps_list) = deps.get(&current) {
            for dep in deps_list {
                let dep_short: &str = &dep.split("::").next().unwrap_or(dep);
                if visited.insert(dep_short.to_string()) {
                    println!("  {} → {}", current, dep_short);
                    queue.push(dep_short.to_string());
                }
            }
        }
    }
}

fn file_to_module(path: &str) -> String {
    let p = Path::new(path);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let parent = p.parent().and_then(|p| p.to_str()).unwrap_or("");
    if parent == "src" {
        stem.to_string()
    } else if parent.starts_with("src/") {
        format!("{}::{}", &parent[4..].replace('/', "::"), stem)
    } else {
        path.to_string()
    }
}

fn extract_deps(source: &str) -> (Vec<String>, Vec<String>) {
    let mut imports = Vec::new();
    let mut submodules = Vec::new();

    for line in source.lines() {
        let line = line.trim();
        if line.starts_with("mod ") && line.ends_with(';') {
            let name = line[4..line.len() - 1].trim();
            if !name.contains('{') { submodules.push(name.to_string()); }
        }
        if line.starts_with("use ") && line.ends_with(';') {
            let use_path = line[4..line.len() - 1].trim();
            if use_path.starts_with("crate::") {
                imports.push(use_path[7..].to_string());
            }
        }
        if line.starts_with("pub use ") && line.ends_with(';') {
            let use_path = line[8..line.len() - 1].trim();
            if use_path.starts_with("crate::") {
                imports.push(use_path[7..].to_string());
            }
        }
    }
    (imports, submodules)
}
