use clap::{Parser, Subcommand};
use log::{error, info};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("xtask started");

    let cli = Cli::parse();

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask 应位于项目根目录下");

    let output_dir = root.join("firmware");
    std::fs::create_dir_all(&output_dir).expect("创建固件输出目录失败");
    let output = output_dir.join("app.bin");

    match cli.cmd {
        Commands::App { args, version } => {
            info!("execute app");
            match args.as_str() {
                "pack" => {
                    info!("构建ota需要的固件包");
                    let cmd = std::process::Command::new("cargo")
                        .current_dir(root)
                        .args([
                            "objcopy",
                            "-p",
                            "app",
                            "--target",
                            "thumbv7em-none-eabihf",
                            "--release",
                            "--",
                            "-O",
                            "binary",
                        ])
                        .arg(&output)
                        .status()
                        .expect("执行失败");
                    if !cmd.success() {
                        error!("执行失败")
                    }
                }
                "run" => {
                    info!("直接运行app，修改app首地址直接烧录并运行app");
                }
                _ => {
                    info!("未知指令")
                }
            }
        }
        Commands::Boot { args } => {
            info!("execute boot")
        }
    }
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Debug, Subcommand, Clone)]
enum Commands {
    App {
        #[arg(long)]
        args: String,
        #[arg(short, long)]
        version: String,
    },
    Boot {
        #[arg(long)]
        args: String,
    },
}
