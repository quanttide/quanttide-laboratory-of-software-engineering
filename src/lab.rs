use std::process::Command;

/// 运行 qtcloud-code review，返回 finding 列表
/// 依赖：`cargo install qtcloud-code-cli` 安装后，`qtcloud-code` 在 PATH 中
pub fn run_review(path: &str, rules: &[&str]) -> Result<Vec<serde_json::Value>, String> {
    let mut cmd = Command::new("qtcloud-code");
    cmd.args(["review", path, "--format", "json", "--rules", &rules.join(",")]);
    let out = cmd.output().map_err(|e| format!("qtcloud-code 执行失败（是否已安装？）: {}", e))?;
    if !out.status.success() {
        let e = String::from_utf8_lossy(&out.stderr);
        return Err(format!("review 返回非零: {}", e));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("解析 finding 失败: {}", e))
}
