use std::process::Command;

const CLI_BIN: &str = "/home/iguo/repos/quanttide/domains/quanttide-code/apps/qtcloud-code/src/cli/target/debug/qtcloud-code";
const PROJ_DIR: &str = "/home/iguo/repos/quanttide/domains/quanttide-code/apps/qtcloud-code/src/cli";

/// 在正式项目上运行 review，返回 finding 列表
pub fn run_review(rules: &[&str]) -> Result<Vec<serde_json::Value>, String> {
    let mut cmd = Command::new(CLI_BIN);
    cmd.args(["review", PROJ_DIR, "--format", "json", "--rules", &rules.join(",")]);
    let out = cmd.output().map_err(|e| format!("运行 review 失败: {}", e))?;
    if !out.status.success() {
        let e = String::from_utf8_lossy(&out.stderr);
        return Err(format!("review 返回非零: {}", e));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("解析 finding 失败: {}", e))
}

/// 查找正式项目中已编译的二进制是否存在
pub fn check_binary() -> bool {
    std::path::Path::new(CLI_BIN).exists()
}

/// 获取正式项目路径
pub fn project_dir() -> &'static str {
    PROJ_DIR
}
