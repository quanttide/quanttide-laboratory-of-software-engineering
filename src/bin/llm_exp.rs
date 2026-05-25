/// 实验：在 qtcloud-code-cli 真实代码上运行所有分析工具

fn main() {
    // 加载真实项目的源码
    let proj = "/home/iguo/repos/quanttide/domains/quanttide-code/apps/qtcloud-code/src/cli/src";
    let files = &[
        "detector/long_function.rs",
        "detector/missing_tests.rs",
        "detector/long_parameter_list.rs",
        "refactor/rename.rs",
        "main.rs",
    ];

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();

    for rel in files {
        let path = format!("{}/{}", proj, rel);
        let Ok(source) = std::fs::read_to_string(&path) else { continue; };
        let Some(tree) = parser.parse(&source, None) else { continue };
        let root = tree.root_node();

        println!("═══ {} ═══", rel);

        // type_info
        let types = qtcloud_code_cli::reflect::analysis::type_info(&source, &tree);
        if !types.is_empty() {
            println!("  变量:");
            for t in &types {
                let typ = t.type_annotation.as_deref().unwrap_or("(推断)");
                println!("    L{} {}: {}", t.line, t.var, typ);
            }
        }

        // call_graph
        let graph = qtcloud_code_cli::reflect::analysis::build_call_graph(&source, &tree);
        if !graph.is_empty() {
            println!("  函数:");
            for (_, n) in &graph {
                let callees = if n.callees.is_empty() { "—".to_string() } else { n.callees.join(", ") };
                let callers = if n.callers.is_empty() { "—".to_string() } else { n.callers.join(", ") };
                if n.callees.len() > 3 || n.callers.len() > 3 {
                    println!("    {}: 调用 {} 个函数 | 被 {} 调用", n.name, n.callees.len(), n.callers.len());
                }
            }

            // 单次调用函数（可内联候选）
            for (_, n) in &graph {
                if n.callees.is_empty() && n.callers.len() == 1 && n.name != "main" {
                    println!("    → 可内联: {} 只被 1 处调用", n.name);
                }
            }
        }

        // code_search: return_err 数量
        let errs = qtcloud_code_cli::reflect::analysis::code_search(&source, &tree, "return_expression");
        if errs.len() > 3 {
            println!("    {} 个 return 表达式", errs.len());
        }
    }
}
