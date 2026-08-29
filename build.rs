use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=replica.toml");
    println!("cargo:rerun-if-changed=dotfiles");

    let mut files = vec![(PathBuf::from("replica.toml"), PathBuf::from("replica.toml"))];
    collect_files(Path::new("dotfiles"), Path::new("dotfiles"), &mut files);
    files.sort_by(|left, right| left.1.cmp(&right.1));

    let mut generated = String::from(
        "pub fn write_embedded_bundle(root: &std::path::Path) -> std::io::Result<()> {\n",
    );
    for (source, relative) in files {
        let absolute = fs::canonicalize(&source)
            .unwrap_or_else(|error| panic!("Could not resolve {}: {error}", source.display()));
        let mode = fs::metadata(&source)
            .unwrap_or_else(|error| panic!("Could not inspect {}: {error}", source.display()))
            .permissions()
            .mode()
            & 0o777;
        generated.push_str(&format!(
            "    super::write_file(root, {:?}, include_bytes!({:?}), {mode:#o})?;\n",
            relative.to_string_lossy(),
            absolute.to_string_lossy(),
        ));
    }
    generated.push_str("    Ok(())\n}\n");

    let output =
        PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set")).join("embedded_bundle.rs");
    fs::write(&output, generated)
        .unwrap_or_else(|error| panic!("Could not write {}: {error}", output.display()));
}

fn collect_files(root: &Path, directory: &Path, files: &mut Vec<(PathBuf, PathBuf)>) {
    let entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("Could not read {}: {error}", directory.display()));
    for entry in entries {
        let entry = entry.expect("directory entry is readable");
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, files);
        } else if path.is_file() {
            let relative = PathBuf::from("dotfiles").join(
                path.strip_prefix(root)
                    .expect("embedded file is below dotfiles"),
            );
            files.push((path, relative));
        }
    }
}
