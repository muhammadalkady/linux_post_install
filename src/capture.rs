//! Capture installed packages, Flatpaks, and enabled user services.

use crate::manifest::{Manifest, PackageProfile};
use crate::os_details::Distro;
use std::collections::BTreeMap;
use std::process::Command;

/// Captures the reproducible package state available through standard CLIs.
pub fn capture(distro: Distro) -> Result<Manifest, String> {
    let packages = capture_packages(distro)?;
    let flatpaks = lines(command_output(
        "flatpak",
        &["list", "--app", "--columns=application"],
    )?);
    let user_services = capture_user_services().unwrap_or_default();
    let key = distro.manifest_key().to_owned();
    let profile = PackageProfile {
        common: Vec::new(),
        packages: BTreeMap::from([(key, packages)]),
    };

    Ok(Manifest {
        selected_profiles: vec!["captured".into()],
        profiles: BTreeMap::from([("captured".into(), profile)]),
        flatpaks,
        user_services,
        ..Manifest::default()
    })
}

fn capture_packages(distro: Distro) -> Result<Vec<String>, String> {
    let (program, args): (&str, Vec<&str>) = match distro {
        Distro::Arch | Distro::Manjaro | Distro::EndeavourOS => ("pacman", vec!["-Qqe"]),
        Distro::Debian
        | Distro::Ubuntu
        | Distro::LinuxMint
        | Distro::PopOS
        | Distro::ElementaryOS
        | Distro::Kali => ("apt-mark", vec!["showmanual"]),
        Distro::Fedora
        | Distro::Rhel
        | Distro::CentOS
        | Distro::RockyLinux
        | Distro::AlmaLinux
        | Distro::AmazonLinux => (
            "dnf",
            vec!["repoquery", "--userinstalled", "--qf", "%{name}\\n"],
        ),
        Distro::OpenSUSE => (
            "zypper",
            vec!["search", "--installed-only", "--type", "package"],
        ),
        Distro::Alpine => ("apk", vec!["info"]),
        Distro::Void => ("xbps-query", vec!["-m"]),
        Distro::Gentoo => ("qlist", vec!["-I"]),
        Distro::Solus => ("eopkg", vec!["list-installed"]),
    };
    Ok(lines(command_output(program, &args)?))
}

fn capture_user_services() -> Result<Vec<String>, String> {
    let output = command_output(
        "systemctl",
        &[
            "--user",
            "list-unit-files",
            "--state=enabled",
            "--no-legend",
            "--plain",
        ],
    )?;
    Ok(output
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_owned)
        .collect())
}

fn command_output(program: &str, args: &[&str]) -> Result<String, String> {
    match Command::new(program).args(args).output() {
        Ok(output) if output.status.success() => String::from_utf8(output.stdout)
            .map_err(|error| format!("Invalid UTF-8 from {program}: {error}")),
        Ok(output) => Err(format!("{program} exited with {}", output.status)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(format!("Could not run {program}: {error}")),
    }
}

fn lines(output: String) -> Vec<String> {
    let mut values: Vec<_> = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect();
    values.sort();
    values.dedup();
    values
}
