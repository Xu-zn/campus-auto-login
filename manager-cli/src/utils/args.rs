use clap::{Parser, Subcommand, Args};

/// 一个示例命令行工具
#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 执行下载任务
    Download(DownloadArgs),
    /// 管理配置文件
    Config(ConfigArgs),
    /// 查看程序运行状态
    Status,
}

/// 下载命令参数
#[derive(Args)]
pub struct DownloadArgs {
    /// 解压后是否删除压缩包
    #[arg(short, long, default_value = "false")]
    pub delete: bool,
}

/// 配置管理参数
#[derive(Args)]
pub struct ConfigArgs {
    /// 创建配置文件
    #[arg(short, long)]
    pub create: bool,
    /// 验证配置文件
    #[arg(short, long)]
    pub validate: bool,
}

