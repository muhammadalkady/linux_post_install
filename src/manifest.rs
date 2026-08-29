//! Declarative machine-replication manifest.

use crate::os_details::Distro;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// A complete machine-replication manifest.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Manifest {
    /// Profiles applied when no profiles are supplied on the command line.
    pub selected_profiles: Vec<String>,
    /// Named groups of packages.
    pub profiles: BTreeMap<String, PackageProfile>,
    /// DankLinux bootstrap configuration.
    pub danklinux: DankLinux,
    /// Flatpak application identifiers.
    pub flatpaks: Vec<String>,
    /// System services to enable and start.
    pub system_services: Vec<String>,
    /// User services to enable and start.
    pub user_services: Vec<String>,
    /// Configuration trees to copy into the user's home directory.
    pub dotfiles: Vec<Dotfile>,
}

/// Packages belonging to one logical profile.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct PackageProfile {
    /// Packages with the same name on every supported distribution.
    pub common: Vec<String>,
    /// Package lists keyed by an `os-release` distribution ID.
    pub packages: BTreeMap<String, Vec<String>>,
}

impl PackageProfile {
    /// Returns the common and distribution-specific packages for `distro`.
    pub fn packages_for(&self, distro: Distro) -> Vec<String> {
        let mut packages = self.common.clone();
        if let Some(specific) = self.packages.get(distro.manifest_key()) {
            packages.extend(specific.iter().cloned());
        }
        packages
    }
}

/// Configuration for the official DankLinux bootstrap installer.
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct DankLinux {
    /// Whether to invoke the installer while applying the manifest.
    pub enabled: bool,
    /// HTTPS URL of the official installer script.
    pub installer_url: String,
}

impl Default for DankLinux {
    fn default() -> Self {
        Self {
            enabled: false,
            installer_url: "https://install.danklinux.com".into(),
        }
    }
}

/// A configuration tree copied from the manifest directory to a target path.
#[derive(Debug, Deserialize, Serialize)]
pub struct Dotfile {
    /// Source file or directory, relative to the manifest unless absolute.
    pub source: String,
    /// Destination path. A leading `~/` expands to the current user's home.
    pub target: String,
    /// Replace `{{HOME}}` in UTF-8 file contents with the current user's home.
    #[serde(default)]
    pub template: bool,
}

impl Manifest {
    /// Reads and parses a TOML manifest.
    pub fn read(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path)
            .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
        toml::from_str(&text)
            .map_err(|error| format!("Invalid manifest {}: {error}", path.display()))
    }

    /// Serializes and writes this manifest as TOML.
    pub fn write(&self, path: &Path) -> Result<(), String> {
        let text = toml::to_string_pretty(self)
            .map_err(|error| format!("Could not serialize manifest: {error}"))?;
        fs::write(path, text)
            .map_err(|error| format!("Could not write {}: {error}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_combines_common_and_distro_packages() {
        let profile = PackageProfile {
            common: vec!["git".into()],
            packages: BTreeMap::from([("fedora".into(), vec!["fd-find".into()])]),
        };

        assert_eq!(profile.packages_for(Distro::Fedora), ["git", "fd-find"]);
        assert_eq!(profile.packages_for(Distro::Arch), ["git"]);
    }
}
