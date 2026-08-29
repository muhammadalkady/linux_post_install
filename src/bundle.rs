//! Extraction of the workstation bundle embedded in the executable.

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded_bundle.rs"));
}

/// A temporary extraction of the embedded manifest and dotfiles.
pub struct ExtractedBundle {
    root: PathBuf,
}

impl ExtractedBundle {
    /// Extracts the embedded bundle and returns its temporary location.
    pub fn extract() -> Result<Self, String> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("System clock error: {error}"))?
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("linux-post-install-{}-{nonce}", std::process::id()));
        fs::create_dir(&root)
            .map_err(|error| format!("Could not create {}: {error}", root.display()))?;
        if let Err(error) = embedded::write_embedded_bundle(&root) {
            let _ = fs::remove_dir_all(&root);
            return Err(format!("Could not extract embedded bundle: {error}"));
        }
        Ok(Self { root })
    }

    /// Returns the extracted manifest path.
    pub fn manifest_path(&self) -> PathBuf {
        self.root.join("replica.toml")
    }
}

impl Drop for ExtractedBundle {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.root) {
            eprintln!(
                "warning: could not remove temporary bundle {}: {error}",
                self.root.display()
            );
        }
    }
}

fn write_file(root: &Path, relative: &str, contents: &[u8], mode: u32) -> io::Result<()> {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, contents)?;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_manifest_assets_and_executable_permissions() {
        let bundle = ExtractedBundle::extract().expect("bundle extracts");
        assert!(bundle.manifest_path().is_file());
        assert!(
            bundle
                .root
                .join("dotfiles/wallpapers/current.jpg")
                .is_file()
        );

        let script = bundle.root.join("dotfiles/hypr/scripts/cleanup.sh");
        let mode = fs::metadata(script)
            .expect("script metadata")
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0);
    }
}
