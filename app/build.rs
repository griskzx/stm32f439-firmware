use std::{env, fs, path::{Path, PathBuf}};

fn main() {
    let manifest_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"),
    );
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is not set"));

    let slot_a = env::var_os("CARGO_FEATURE_SLOT_A").is_some();
    let slot_b = env::var_os("CARGO_FEATURE_SLOT_B").is_some();
    let memory_script = match (slot_a, slot_b) {
        (true, false) => "memory-a.x",
        (false, true) => "memory-b.x",
        _ => panic!("select exactly one app slot: slot-a or slot-b"),
    };

    copy_script(&manifest_dir.join(memory_script), &out_dir.join("memory.x"));
    copy_script(&manifest_dir.join("ccm.x"), &out_dir.join("ccm.x"));

    println!("cargo:rerun-if-changed={memory_script}");
    println!("cargo:rerun-if-changed=ccm.x");

    println!("cargo:rustc-link-search={}", out_dir.display());
}

fn copy_script(source: &Path, destination: &Path) {
    fs::copy(source, destination)
        .unwrap_or_else(|error| panic!("failed to copy {source:?}: {error}"));
}
