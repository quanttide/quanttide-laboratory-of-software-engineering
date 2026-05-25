/// reflect 实验：跨 finding 根因分析
/// 把整个项目的所有 finding 喂给 LLM，让它发现跨文件的重复模式

use std::process::Command;

#[tokio::main]
async fn main() {
    let api_key = match qtcloud_code_cli::llm::get_api_key_from_vault().await {
        Ok(k) => k,
        Err(e) => { eprintln!("Vault 失败: {}", e); std::process::exit(1); }
    };

    // 1. 在正式项目上跑 review，收集全部 finding
    let bin = "/home/iguo/repos/quanttide/domains/quanttide-code/apps/qtcloud-code/src/cli/target/debug/qtcloud-code";
    let proj = "/home/iguo/repos/quanttide/domains/quanttide-code/apps/qtcloud-code/src/cli";
    let out = Command::new(bin)
        .args(["review", proj, "--format", "json", "--rules", "long-function,long-parameter-list"])
        .output()
        .expect("review 失败");

    let findings: Vec<serde_json::Value> = serde_json::from_slice(&out.stdout).unwrap_or_default();

    // 2. 准备 LLM 输入：按模块分组统计
    let mut by_module: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
    for f in &findings {
        let file = f["file"].as_str().unwrap_or("");
        let msg = f["message"].as_str().unwrap_or("");
        let module = file.rsplitn(2, '/').nth(1).unwrap_or(file);
        by_module.entry(module.to_string()).or_default().push(msg.to_string());
    }

    println!("=== review: {} 个 finding，分布在 {} 个模块 ===", findings.len(), by_module.len());
    for (m, fs) in &by_module {
        println!("  {}: {} 条", m, fs.len());
    }

    // 3. reflect: LLM 跨 finding 根因分析
    let prompt = format!(
        r#"你是一个代码架构师。以下是项目的全部 review finding，各自包含文件路径和问题描述。

请做根因分析：
1. 不同文件的 finding 是否有共同模式？比如都集中在某一层？
2. 从架构层面看，这些问题的源头是什么？
3. 解决这些问题的根本措施是什么？

Finding 列表：
{}"#,
        by_module.iter().map(|(m, fs)| {
            format!("  {}:\n{}", m, fs.iter().map(|f| format!("    - {}", f)).collect::<Vec<_>>().join("\n"))
        }).collect::<Vec<_>>().join("\n\n")
    );

    println!("\n=== reflect 跨 finding 根因分析 ===");
    match qtcloud_code_cli::llm::enhance_finding("", &prompt, &api_key).await {
        Ok(enh) => println!("{}", enh.explanation),
        Err(e) => eprintln!("LLM 错误: {}", e),
    }
}
