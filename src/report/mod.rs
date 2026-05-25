use std::collections::BTreeMap;
use std::io::Write;
use std::time::SystemTime;

use crate::detect::{Finding, Severity};

macro_rules! writeln_err {
    ($dst:expr $(, $arg:expr)* $(,)?) => {
        writeln!($dst $(, $arg)*).map_err(|e| e.to_string())
    };
}

pub fn write_json<W: Write>(writer: &mut W, findings: &[Finding]) -> Result<(), String> {
    let output: Vec<serde_json::Value> = findings
        .iter()
        .map(|f| {
            serde_json::json!({
                "file": f.file_path,
                "line": f.line,
                "column": f.column,
                "severity": format!("{:?}", f.severity).to_uppercase(),
                "rule_id": f.rule_id,
                "message": f.message,
            })
        })
        .collect();
    let json = serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?;
    writeln_err!(writer, "{}", json)
}

pub fn write_terminal<W: Write>(writer: &mut W, findings: &[Finding]) -> Result<(), String> {
    if findings.is_empty() {
        writeln_err!(writer, "未发现问题")?;
        return Ok(());
    }

    for f in findings {
        let (icon, tag) = match f.severity {
            Severity::Must => ("🔴", "MUST"),
            Severity::Should => ("🟡", "SHOULD"),
            Severity::May => ("🔵", "MAY"),
        };
        writeln_err!(
            writer,
            "{} [{}] {}:{}  {}  {}",
            icon, tag, f.file_path.display(), f.line, f.rule_id, f.message
        )?;
    }
    Ok(())
}

pub fn write_status<W: Write>(writer: &mut W, findings: &[Finding]) -> Result<(), String> {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    writeln_err!(writer, "# Code Scan Status")?;
    writeln_err!(writer)?;
    writeln_err!(writer, "> 自动生成于 qtcloud-code review，时间戳: {}", timestamp)?;
    writeln_err!(writer)?;

    write_status_summary(writer, findings)?;

    if findings.is_empty() {
        writeln_err!(writer, "✅ 未发现问题")?;
        return Ok(());
    }

    write_status_details(writer, findings)
}

fn write_status_summary<W: Write>(writer: &mut W, findings: &[Finding]) -> Result<(), String> {
    let total = findings.len();
    let must = findings.iter().filter(|f| f.severity == Severity::Must).count();
    let should = findings.iter().filter(|f| f.severity == Severity::Should).count();
    let may = findings.iter().filter(|f| f.severity == Severity::May).count();

    writeln_err!(writer, "## 汇总")?;
    writeln_err!(writer)?;
    writeln_err!(writer, "| 级别 | 数量 |")?;
    writeln_err!(writer, "|------|------|")?;
    writeln_err!(writer, "| MUST   | {} |", must)?;
    writeln_err!(writer, "| SHOULD | {} |", should)?;
    writeln_err!(writer, "| MAY    | {} |", may)?;
    writeln_err!(writer, "| **Total** | **{}** |", total)?;
    writeln_err!(writer)
}

fn write_status_details<W: Write>(writer: &mut W, findings: &[Finding]) -> Result<(), String> {
    writeln_err!(writer, "## 详情")?;
    writeln_err!(writer)?;

    let mut by_file: BTreeMap<String, Vec<&Finding>> = BTreeMap::new();
    for f in findings {
        let key = f.file_path.to_string_lossy().to_string();
        by_file.entry(key).or_default().push(f);
    }

    for (file, findings) in &by_file {
        writeln_err!(writer, "- **{}** ({} 项)", file, findings.len())?;
        for f in findings {
            let (icon, tag) = match f.severity {
                Severity::Must => ("🔴", "MUST"),
                Severity::Should => ("🟡", "SHOULD"),
                Severity::May => ("🔵", "MAY"),
            };
            writeln_err!(writer, "  - {} **{}** `{}` {}:{} — {}", icon, tag, f.rule_id, f.line, f.column, f.message)?;
        }
    }

    Ok(())
}
