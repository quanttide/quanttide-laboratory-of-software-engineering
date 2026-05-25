use std::io::{self};
use std::path::{Path, PathBuf};
use std::process;

use clap::{Parser, Subcommand};

use qtcloud_code_cli::detect::{Detector, Finding};
use qtcloud_code_cli::lang::LanguageParser;

#[derive(Parser)]
#[command(name = "qtcloud-code", about = "多语言代码静态分析与质量检测")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 扫描目录中的代码文件，检测问题
    Scan {
        /// 目标目录
        path: String,
        #[arg(long, default_value = "terminal")]
        format: String,
        /// 将扫描结果写入被检测项目的 STATUS.md
        #[arg(long)]
        status: bool,
    },
    /// 列出可用检测规则
    ListRules,
}

fn list_detectors() -> Vec<Box<dyn Detector>> {
    vec![
        Box::new(qtcloud_code_cli::detect::unsafe_block::UnsafeBlockDetector),
        Box::new(qtcloud_code_cli::detect::long_function::LongFunctionDetector),
    ]
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Scan { path, format, status } => run_scan(&path, &format, status),
        Commands::ListRules => run_list_rules(),
    };

    if let Err(e) = result {
        eprintln!("错误: {}", e);
        process::exit(1);
    }
}

fn run_scan(path: &str, format: &str, write_status: bool) -> Result<(), String> {
    let root = Path::new(path);
    if !root.exists() {
        return Err(format!("路径不存在: {}", path));
    }

    let mut parser = qtcloud_code_cli::lang::rust::RustParser::new()?;
    let detectors = list_detectors();
    let mut all_findings: Vec<Finding> = Vec::new();

    let entries = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file());

    let extensions = parser.file_extensions();
    for entry in entries {
        let file_path = entry.path();
        if !extensions
            .iter()
            .any(|ext| file_path.extension().map_or(false, |e| e == *ext))
        {
            continue;
        }

        let source = match std::fs::read_to_string(file_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("跳过 {}: {}", file_path.display(), e);
                continue;
            }
        };

        let Some(result) = parser.parse(file_path, &source) else {
            eprintln!("跳过 {}: 解析失败", file_path.display());
            continue;
        };

        for detector in &detectors {
            let findings = detector.detect(&result.source, &result.tree, &file_path.to_path_buf());
            all_findings.extend(findings);
        }
    }

    match format {
        "json" => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            qtcloud_code_cli::report::write_json(&mut handle, &all_findings)?;
        }
        _ => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            qtcloud_code_cli::report::write_terminal(&mut handle, &all_findings)?;
        }
    }

    if write_status {
        let status_path = find_project_root(root).map(|p| p.join("STATUS.md"));
        if let Some(status_path) = status_path {
            let file = std::fs::File::create(&status_path)
                .map_err(|e| format!("无法创建 STATUS.md: {}", e))?;
            let mut writer = std::io::BufWriter::new(file);
            qtcloud_code_cli::report::write_status(&mut writer, &all_findings)?;
            println!("\nSTATUS.md 已写入: {}", status_path.display());
        } else {
            eprintln!("警告: 未找到项目根目录（Cargo.toml），跳过 STATUS.md 写入");
        }
    }

    Ok(())
}

fn find_project_root(path: &Path) -> Option<PathBuf> {
    let mut current = Some(path.to_path_buf());
    while let Some(dir) = current {
        if dir.join("Cargo.toml").exists() {
            return Some(dir);
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }
    None
}

fn run_list_rules() -> Result<(), String> {
    println!("可用检测规则:");
    for d in list_detectors() {
        println!("  {} — {}", d.rule_id(), d.description());
    }
    Ok(())
}
