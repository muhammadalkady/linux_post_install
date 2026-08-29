//! Linux distribution detection and package-manager selection.
//!
//! This module reads an [`os-release`](https://www.freedesktop.org/software/systemd/man/latest/os-release.html)
//! file, identifies the distribution from its `ID` field, and maps that
//! distribution to its native package manager.

use std::fs;

/// Details detected from an `os-release` file.
#[derive(Debug)]
pub struct OSDetails {
    /// A human-readable summary containing the OS name, version, and release type.
    pub name: String,
    /// The detected Linux distribution.
    pub distro: Distro,
}

/// A supported Linux distribution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Distro {
    /// Arch Linux.
    Arch,
    /// Manjaro Linux.
    Manjaro,
    /// EndeavourOS.
    EndeavourOS,
    /// Debian.
    Debian,
    /// Ubuntu.
    Ubuntu,
    /// Linux Mint.
    LinuxMint,
    /// Pop!_OS.
    PopOS,
    /// elementary OS.
    ElementaryOS,
    /// Kali Linux.
    Kali,
    /// Fedora Linux.
    Fedora,
    /// Red Hat Enterprise Linux.
    Rhel,
    /// CentOS.
    CentOS,
    /// Rocky Linux.
    RockyLinux,
    /// AlmaLinux.
    AlmaLinux,
    /// Amazon Linux.
    AmazonLinux,
    /// openSUSE or SUSE Linux Enterprise Server.
    OpenSUSE,
    /// Alpine Linux.
    Alpine,
    /// Void Linux.
    Void,
    /// Gentoo Linux.
    Gentoo,
    /// Solus.
    Solus,
}

/// A package manager used by one or more supported distributions.
#[derive(Debug)]
pub enum PackageManager {
    /// APT, invoked through `apt-get`.
    Apt,
    /// The Arch Linux `pacman` package manager.
    Pacman,
    /// The DNF package manager.
    Dnf,
    /// The openSUSE `zypper` package manager.
    Zypper,
    /// The Alpine Package Keeper.
    Apk,
    /// The XBPS package manager.
    Xbps,
    /// Gentoo's Portage package manager.
    Portage,
    /// The Solus `eopkg` package manager.
    Eopkg,
}

impl PackageManager {
    /// Returns the executable used to invoke this package manager.
    pub fn executable(&self) -> &str {
        match self {
            PackageManager::Apt => "apt-get",
            PackageManager::Pacman => "pacman",
            PackageManager::Dnf => "dnf",
            PackageManager::Zypper => "zypper",
            PackageManager::Apk => "apk",
            PackageManager::Xbps => "xbps-install",
            PackageManager::Portage => "emerge",
            PackageManager::Eopkg => "eopkg",
        }
    }

    /// Returns the arguments that precede package names during installation.
    ///
    /// Portage needs no explicit install subcommand, so its slice is empty.
    pub fn install_args(&self) -> &'static [&'static str] {
        match self {
            PackageManager::Apt
            | PackageManager::Dnf
            | PackageManager::Zypper
            | PackageManager::Eopkg => &["install"],
            PackageManager::Pacman => &["-S", "--needed"],
            PackageManager::Apk => &["add"],
            PackageManager::Xbps => &["-S"],
            PackageManager::Portage => &[],
        }
    }
}

impl Distro {
    /// Returns the manifest key used for distribution-specific package lists.
    pub fn manifest_key(&self) -> &'static str {
        match self {
            Distro::Arch => "arch",
            Distro::Manjaro => "manjaro",
            Distro::EndeavourOS => "endeavouros",
            Distro::Debian => "debian",
            Distro::Ubuntu => "ubuntu",
            Distro::LinuxMint => "linuxmint",
            Distro::PopOS => "pop",
            Distro::ElementaryOS => "elementary",
            Distro::Kali => "kali",
            Distro::Fedora => "fedora",
            Distro::Rhel => "rhel",
            Distro::CentOS => "centos",
            Distro::RockyLinux => "rocky",
            Distro::AlmaLinux => "almalinux",
            Distro::AmazonLinux => "amzn",
            Distro::OpenSUSE => "opensuse",
            Distro::Alpine => "alpine",
            Distro::Void => "void",
            Distro::Gentoo => "gentoo",
            Distro::Solus => "solus",
        }
    }

    /// Returns the package manager associated with this distribution.
    pub fn package_manager(&self) -> PackageManager {
        match self {
            Distro::Arch | Distro::Manjaro | Distro::EndeavourOS => PackageManager::Pacman,
            Distro::Debian
            | Distro::Ubuntu
            | Distro::LinuxMint
            | Distro::PopOS
            | Distro::ElementaryOS
            | Distro::Kali => PackageManager::Apt,
            Distro::Fedora
            | Distro::Rhel
            | Distro::CentOS
            | Distro::RockyLinux
            | Distro::AlmaLinux
            | Distro::AmazonLinux => PackageManager::Dnf,
            Distro::OpenSUSE => PackageManager::Zypper,
            Distro::Alpine => PackageManager::Apk,
            Distro::Void => PackageManager::Xbps,
            Distro::Gentoo => PackageManager::Portage,
            Distro::Solus => PackageManager::Eopkg,
        }
    }
}

impl TryFrom<&str> for Distro {
    type Error = String;

    fn try_from(id: &str) -> Result<Self, Self::Error> {
        match id {
            "arch" => Ok(Self::Arch),
            "manjaro" => Ok(Self::Manjaro),
            "endeavouros" => Ok(Self::EndeavourOS),
            "debian" => Ok(Self::Debian),
            "ubuntu" => Ok(Self::Ubuntu),
            "linuxmint" => Ok(Self::LinuxMint),
            "pop" => Ok(Self::PopOS),
            "elementary" => Ok(Self::ElementaryOS),
            "kali" => Ok(Self::Kali),
            "fedora" => Ok(Self::Fedora),
            "rhel" => Ok(Self::Rhel),
            "centos" => Ok(Self::CentOS),
            "rocky" => Ok(Self::RockyLinux),
            "almalinux" => Ok(Self::AlmaLinux),
            "amzn" => Ok(Self::AmazonLinux),
            "opensuse" | "opensuse-leap" | "opensuse-tumbleweed" | "sles" => Ok(Self::OpenSUSE),
            "alpine" => Ok(Self::Alpine),
            "void" => Ok(Self::Void),
            "gentoo" => Ok(Self::Gentoo),
            "solus" => Ok(Self::Solus),
            _ => Err(format!("Unsupported Linux distribution: {id}")),
        }
    }
}

/// Reads an `os-release` file and returns its detected OS details.
///
/// The `ID` field determines the [`Distro`]. The display name combines the
/// optional `NAME`, `VERSION`, and `RELEASE_TYPE` fields; missing optional
/// fields are represented by empty strings.
///
/// # Errors
///
/// Returns an error when the file cannot be read, the required `ID` field is
/// missing, or its value does not identify a supported distribution.
pub fn get_os_details(os_release_path: &str) -> Result<OSDetails, String> {
    let content = fs::read_to_string(os_release_path)
        .map_err(|error| format!("Could not read {os_release_path}: {error}"))?;

    let name = format!(
        "OS: {}, {}, {}",
        find_value(&content, "NAME").unwrap_or_default(),
        find_value(&content, "VERSION").unwrap_or_default(),
        find_value(&content, "RELEASE_TYPE").unwrap_or_default(),
    );
    let id = find_value(&content, "ID").ok_or("Missing ID in os-release")?;
    let distro = Distro::try_from(id)?;

    Ok(OSDetails { name, distro })
}

/// Finds a value by key in `os-release` content and removes surrounding quotes.
fn find_value<'a>(content: &'a str, searched_key: &str) -> Option<&'a str> {
    for line in content.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        if key == searched_key {
            return Some(value.trim_matches('"'));
        }
    }
    None
}
