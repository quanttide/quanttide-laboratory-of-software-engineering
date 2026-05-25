use clap::{Parser, Subcommand};

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
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Scan { path } => {
            println!("扫描目录: {}", path);
        }
    }
}
