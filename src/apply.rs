//! Manifest planning and application.

use crate::manifest::{Dotfile, Manifest};
use crate::os_details::Distro;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Applies the selected parts of a manifest, or prints them in dry-run mode.
pub fn apply(
    manifest: &Manifest,
    manifest_path: &Path,
    distro: Distro,
    profiles: &[String],
    dry_run: bool,
) -> Result<(), String> {
    let selected = if profiles.is_empty() {
        &manifest.selected_profiles
    } else {
        profiles
    };

    let mut packages = BTreeSet::new();
    for name in selected {
        let profile = manifest
            .profiles
            .get(name)
            .ok_or_else(|| format!("Unknown package profile: {name}"))?;
        packages.extend(profile.packages_for(distro));
    }

    if !packages.is_empty() {
        let manager = distro.package_manager();
        let mut args = vec![manager.executable().to_owned()];
        args.extend(manager.install_args().iter().map(|arg| (*arg).to_owned()));
        args.extend(packages);
        run("sudo", &args, dry_run)?;
    }

    let base = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    install_systemd_units(base, dry_run)?;
    install_claude_code(dry_run)?;

    if manifest.danklinux.enabled {
        if !manifest.danklinux.installer_url.starts_with("https://") {
            return Err("DankLinux installer_url must use HTTPS".into());
        }
        let script = format!(
            "curl -fsSL {} | sh",
            shell_quote(&manifest.danklinux.installer_url)
        );
        run("sh", &["-c".into(), script], dry_run)?;
    }

    if !manifest.flatpaks.is_empty() {
        let mut args = vec!["install".into(), "-y".into(), "flathub".into()];
        args.extend(manifest.flatpaks.iter().cloned());
        run("flatpak", &args, dry_run)?;
    }

    for service in &manifest.system_services {
        run(
            "sudo",
            &[
                "systemctl".into(),
                "enable".into(),
                "--now".into(),
                service.clone(),
            ],
            dry_run,
        )?;
    }
    for service in &manifest.user_services {
        run(
            "systemctl",
            &[
                "--user".into(),
                "enable".into(),
                "--now".into(),
                service.clone(),
            ],
            dry_run,
        )?;
    }

    for dotfile in &manifest.dotfiles {
        copy_dotfile(dotfile, base, dry_run)?;
    }
    Ok(())
}

/// Installs and enables every `*.service` unit tracked in `dotfiles/systemd`.
///
/// `system_services` in the manifest can only enable units that already
/// exist on the target, since dotfile copies and most manifest steps run
/// without root. These unit files need an explicit `sudo cp` first.
fn install_systemd_units(base: &Path, dry_run: bool) -> Result<(), String> {
    let units_dir = base.join("dotfiles/systemd");
    if !units_dir.is_dir() {
        return Ok(());
    }

    let mut units: Vec<PathBuf> = fs::read_dir(&units_dir)
        .map_err(|error| format!("Could not read {}: {error}", units_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "service"))
        .collect();
    units.sort();
    if units.is_empty() {
        return Ok(());
    }

    for unit in &units {
        let name = unit.file_name().unwrap().to_string_lossy().into_owned();
        let target = format!("/etc/systemd/system/{name}");
        run(
            "sudo",
            &["cp".into(), unit.to_string_lossy().into_owned(), target],
            dry_run,
        )?;
    }
    run(
        "sudo",
        &["systemctl".into(), "daemon-reload".into()],
        dry_run,
    )?;
    for unit in &units {
        let name = unit.file_name().unwrap().to_string_lossy().into_owned();
        run(
            "sudo",
            &["systemctl".into(), "enable".into(), "--now".into(), name],
            dry_run,
        )?;
    }
    Ok(())
}

/// Installs Claude Code via the official installer, unless it is already on
/// `PATH`. Not a distro package, so it can't live in a package profile.
fn install_claude_code(dry_run: bool) -> Result<(), String> {
    let already_installed = Command::new("sh")
        .args(["-c", "command -v claude"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    if already_installed {
        println!("Claude Code already installed, skipping");
        return Ok(());
    }
    run(
        "sh",
        &[
            "-c".into(),
            "curl -fsSL https://claude.ai/install.sh | bash".into(),
        ],
        dry_run,
    )
}

fn run(program: &str, args: &[String], dry_run: bool) -> Result<(), String> {
    let display = std::iter::once(program)
        .chain(args.iter().map(String::as_str))
        .map(shell_quote)
        .collect::<Vec<_>>()
        .join(" ");
    println!("{} {display}", if dry_run { "DRY-RUN" } else { "RUN" });
    if dry_run {
        return Ok(());
    }

    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|error| format!("Could not run {program}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Command failed with {status}: {display}"))
    }
}

fn copy_dotfile(dotfile: &Dotfile, base: &Path, dry_run: bool) -> Result<(), String> {
    let source = if Path::new(&dotfile.source).is_absolute() {
        PathBuf::from(&dotfile.source)
    } else {
        base.join(&dotfile.source)
    };
    let target = expand_home(&dotfile.target)?;
    println!(
        "{} copy {} -> {}",
        if dry_run { "DRY-RUN" } else { "RUN" },
        source.display(),
        target.display()
    );
    if dry_run {
        return Ok(());
    }
    copy_recursively(&source, &target, dotfile.template)
}

fn copy_recursively(source: &Path, target: &Path, template: bool) -> Result<(), String> {
    if source.is_dir() {
        fs::create_dir_all(target)
            .map_err(|error| format!("Could not create {}: {error}", target.display()))?;
        for entry in fs::read_dir(source)
            .map_err(|error| format!("Could not read {}: {error}", source.display()))?
        {
            let entry = entry.map_err(|error| error.to_string())?;
            copy_recursively(&entry.path(), &target.join(entry.file_name()), template)?;
        }
    } else if source.is_file() {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
        }
        if template {
            let content = fs::read_to_string(source).map_err(|error| {
                format!("Could not read template {}: {error}", source.display())
            })?;
            let home = env::var("HOME").map_err(|_| "HOME is not valid UTF-8")?;
            fs::write(target, content.replace("{{HOME}}", &home)).map_err(|error| {
                format!("Could not write template {}: {error}", target.display())
            })?;
        } else {
            fs::copy(source, target).map_err(|error| {
                format!(
                    "Could not copy {} to {}: {error}",
                    source.display(),
                    target.display()
                )
            })?;
        }
    } else {
        return Err(format!(
            "Dotfile source does not exist: {}",
            source.display()
        ));
    }
    Ok(())
}

fn expand_home(path: &str) -> Result<PathBuf, String> {
    if path == "~" || path.starts_with("~/") {
        let home = env::var_os("HOME").ok_or("HOME is not set")?;
        return Ok(PathBuf::from(home).join(path.trim_start_matches("~/")));
    }
    Ok(PathBuf::from(path))
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "-_=+.,/:@".contains(character))
    {
        value.to_owned()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

        #[test]
        fn shell_quote_round_trips_through_a_real_shell(value in ".{0,40}") {
            prop_assume!(!value.contains('\0'));
            let quoted = shell_quote(&value);
            let output = Command::new("sh")
                .arg("-c")
                .arg(format!("printf '%s' {quoted}"))
                .output()
                .expect("sh runs");
            prop_assert!(output.status.success());
            prop_assert_eq!(String::from_utf8_lossy(&output.stdout), value);
        }

        #[test]
        fn expand_home_leaves_non_tilde_paths_unchanged(path in ".{0,40}") {
            prop_assume!(!path.contains('\0'));
            prop_assume!(path != "~" && !path.starts_with("~/"));
            prop_assert_eq!(expand_home(&path).unwrap(), PathBuf::from(&path));
        }

        #[test]
        fn expand_home_joins_home_for_tilde_paths(rest in "[a-zA-Z0-9/_.-]{0,20}") {
            let home = env::var_os("HOME").expect("HOME is set in test environment");
            let path = format!("~/{rest}");
            prop_assert_eq!(expand_home(&path).unwrap(), PathBuf::from(home).join(&rest));
        }
    }
}
