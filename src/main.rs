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
    /// 审查目录中的代码文件，检测问题
    Review {
        /// 目标目录
        path: String,
        #[arg(long, default_value = "terminal")]
        format: String,
        /// 仅运行指定的检测规则（逗号分隔）
        #[arg(long, value_delimiter = ',')]
        rules: Option<Vec<String>>,
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
        Box::new(qtcloud_code_cli::detect::long_parameter_list::LongParameterListDetector),
    ]
}

fn all_rule_ids() -> Vec<&'static str> {
    let mut ids: Vec<&str> = list_detectors().iter().map(|d| d.rule_id()).collect();
    ids.push(qtcloud_code_cli::detect::unused_variable::RULE_ID);
    ids
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Review { path, format, rules, status } => run_review(&path, &format, rules, status),
        Commands::ListRules => run_list_rules(),
    };

    if let Err(e) = result {
        eprintln!("错误: {}", e);
        process::exit(1);
    }
}

fn run_review(path: &str, format: &str, cli_rules: Option<Vec<String>>, write_status: bool) -> Result<(), String> {
    let raw_path = Path::new(path);
    if !raw_path.exists() {
        return Err(format!("路径不存在: {}", path));
    }
    let root = raw_path
        .canonicalize()
        .map_err(|e| format!("无法规范化路径: {}", e))?;

    let config = qtcloud_code_cli::config::load_contract(&root);
    let enabled_rules = qtcloud_code_cli::config::resolve_enabled_rules(&cli_rules, &config, &all_rule_ids());
    let all_detectors = list_detectors();
    let detectors: Vec<Box<dyn Detector>> = all_detectors.into_iter().filter(|d| enabled_rules.contains(&d.rule_id().to_string())).collect();

    let mut parsers: Vec<Box<dyn LanguageParser>> = vec![
        Box::new(qtcloud_code_cli::lang::rust::RustParser::new()?),
        Box::new(qtcloud_code_cli::lang::python::PythonParser::new()?),
        Box::new(qtcloud_code_cli::lang::go::GoParser::new()?),
        Box::new(qtcloud_code_cli::lang::dart::DartParser::new()?),
        Box::new(qtcloud_code_cli::lang::typescript::TypeScriptParser::new()?),
        Box::new(qtcloud_code_cli::lang::typescript::TsxParser::new()?),
    ];
    let mut all_findings: Vec<Finding> = Vec::new();

    let entries = walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file());

    for entry in entries {
        let file_path = entry.path();
        let Some(ext) = file_path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        let Some(parser) = parsers.iter_mut().find(|p| p.file_extensions().contains(&ext)) else {
            continue;
        };

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

    if let Some(project_root) = find_project_root(&root) {
        let compiler_findings =
            qtcloud_code_cli::detect::unused_variable::check_compiler(&project_root, &enabled_rules)?;
        all_findings.extend(compiler_findings);
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
        let status_path = find_project_root(&root).map(|p| p.join("STATUS.md"));
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
    println!("可用检测规则（语法级）:");
    for d in list_detectors() {
        println!("  {} — {}", d.rule_id(), d.description());
    }
    println!("\n可用检测规则（编译器级）:");
    println!("  {} — {}", qtcloud_code_cli::detect::unused_variable::RULE_ID, qtcloud_code_cli::detect::unused_variable::DESCRIPTION);
    Ok(())
}
