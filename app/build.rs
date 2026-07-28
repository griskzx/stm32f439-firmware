use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is not set"));

    for script in ["memory.x", "ccm.x"] {
        fs::copy(script, out_dir.join(script))
            .unwrap_or_else(|error| panic!("failed to copy {script}: {error}"));
        println!("cargo:rerun-if-changed={script}");
    }

    println!("cargo:rustc-link-search={}", out_dir.display());
}
