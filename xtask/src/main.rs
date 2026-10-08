use std::{env, error::Error, fs, io, path::Path, process::Command};

const TARGET: &str = "thumbv7em-none-eabihf";
const USAGE: &str = "Usage: cargo run -p xtask -- build [app|boot|all]\n\
                    Builds release firmware and writes BIN files to target/firmware/.";

fn main() {
    if let Err(error) = run() {
        eprintln!("xtask: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let packages: &[&str] = match args.as_slice() {
        [] | ["help" | "--help" | "-h"] => {
            println!("{USAGE}");
            return Ok(());
        }
        ["build"] | ["build", "all"] => &["boot", "app"],
        ["build", "app"] => &["app"],
        ["build", "boot"] => &["boot"],
        _ => return Err(io::Error::other(format!("invalid arguments\n{USAGE}")).into()),
    };

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| io::Error::other("cannot locate workspace"))?;
    let output_dir = workspace.join("target/firmware");
    fs::create_dir_all(&output_dir)?;

    for package in packages {
        build_firmware(workspace, &output_dir, package)?;
    }
    Ok(())
}

fn build_firmware(
    workspace: &Path,
    output_dir: &Path,
    package: &str,
) -> Result<(), Box<dyn Error>> {
    let output = output_dir.join(format!("{package}.bin"));
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    println!("Building {package} -> {}", output.display());

    // Cargo loads configuration from the working directory and its ancestors.
    // Each firmware package has its own .cargo/config.toml with linker flags.
    let status = Command::new(cargo)
        .current_dir(workspace.join(package))
        .args([
            "objcopy",
            "--release",
            "--package",
            package,
            "--bin",
            package,
            "--target",
            TARGET,
            "--",
            "-O",
            "binary",
        ])
        .arg(&output)
        .status()?;

    if !status.success() {
        return Err(io::Error::other(format!("building {package} failed ({status})")).into());
    }

    println!(
        "Generated {} ({} bytes)",
        output.display(),
        fs::metadata(&output)?.len()
    );
    Ok(())
}
