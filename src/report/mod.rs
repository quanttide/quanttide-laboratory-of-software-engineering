use std::collections::BTreeMap;
use std::io::Write;
use std::time::SystemTime;

use crate::detect::{Finding, Severity};

pub fn write_json<W: Write>(writer: &mut W, findings: &[Finding]) -> Result<(), String> {
    let output: Vec<serde_json::Value> = findings
        .iter()
        .map(|f| {
            serde_json::json!({
                "file": f.file_path,
                "line": f.line,
                "column": f.column,
                "severity": format!("{:?}", f.severity).to_lowercase(),
                "rule_id": f.rule_id,
                "message": f.message,
            })
        })
        .collect();
    let json = serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?;
    writeln!(writer, "{}", json).map_err(|e| e.to_string())
}

pub fn write_terminal<W: Write>(writer: &mut W, findings: &[Finding]) -> Result<(), String> {
    if findings.is_empty() {
        writeln!(writer, "未发现问题").map_err(|e| e.to_string())?;
        return Ok(());
    }

    let mut by_file: BTreeMap<String, Vec<&Finding>> = BTreeMap::new();
    for f in findings {
        let key = f.file_path.to_string_lossy().to_string();
        by_file.entry(key).or_default().push(f);
    }

    for (file, findings) in &by_file {
        writeln!(writer, "{}:", file).map_err(|e| e.to_string())?;
        for f in findings {
            let icon = match f.severity {
                crate::detect::Severity::Error => "🔴",
                crate::detect::Severity::Warning => "🟡",
                crate::detect::Severity::Info => "🔵",
            };
            writeln!(
                writer,
                "  {} {}:{}  {}  {}",
                icon, f.line, f.column, f.rule_id, f.message
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub fn write_status<W: Write>(writer: &mut W, findings: &[Finding]) -> Result<(), String> {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let total = findings.len();
    let warnings = findings.iter().filter(|f| f.severity == Severity::Warning).count();
    let errors = findings.iter().filter(|f| f.severity == Severity::Error).count();
    let infos = findings.iter().filter(|f| f.severity == Severity::Info).count();

    writeln!(writer, "# Code Scan Status").map_err(|e| e.to_string())?;
    writeln!(writer).map_err(|e| e.to_string())?;
    writeln!(writer, "> 自动生成于 qtcloud-code scan，时间戳: {}", timestamp).map_err(|e| e.to_string())?;
    writeln!(writer).map_err(|e| e.to_string())?;
    writeln!(writer, "## 汇总").map_err(|e| e.to_string())?;
    writeln!(writer).map_err(|e| e.to_string())?;
    writeln!(writer, "| 严重度 | 数量 |").map_err(|e| e.to_string())?;
    writeln!(writer, "|--------|------|").map_err(|e| e.to_string())?;
    writeln!(writer, "| Error   | {} |", errors).map_err(|e| e.to_string())?;
    writeln!(writer, "| Warning | {} |", warnings).map_err(|e| e.to_string())?;
    writeln!(writer, "| Info    | {} |", infos).map_err(|e| e.to_string())?;
    writeln!(writer, "| **Total** | **{}** |", total).map_err(|e| e.to_string())?;
    writeln!(writer).map_err(|e| e.to_string())?;

    if findings.is_empty() {
        writeln!(writer, "✅ 未发现问题").map_err(|e| e.to_string())?;
        return Ok(());
    }

    writeln!(writer, "## 详情").map_err(|e| e.to_string())?;
    writeln!(writer).map_err(|e| e.to_string())?;

    let mut by_file: BTreeMap<String, Vec<&Finding>> = BTreeMap::new();
    for f in findings {
        let key = f.file_path.to_string_lossy().to_string();
        by_file.entry(key).or_default().push(f);
    }

    for (file, findings) in &by_file {
        writeln!(writer, "- **{}** ({} 项)", file, findings.len()).map_err(|e| e.to_string())?;
        for f in findings {
            let icon = match f.severity {
                Severity::Error => "🔴",
                Severity::Warning => "🟡",
                Severity::Info => "🔵",
            };
            writeln!(writer, "  - {} `{}` {}:{} — {}", icon, f.rule_id, f.line, f.column, f.message)
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
