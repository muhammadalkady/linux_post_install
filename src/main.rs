mod apply;
mod capture;
mod manifest;
mod os_details;

use crate::manifest::Manifest;
use crate::os_details::get_os_details;
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const OS_RELEASE_PATH: &str = "/etc/os-release";
const DEFAULT_MANIFEST_PATH: &str = "replica.toml";

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        print_help();
        return Ok(());
    };
    if matches!(command, "help" | "--help" | "-h") {
        print_help();
        return Ok(());
    }

    let os = get_os_details(OS_RELEASE_PATH)?;
    println!("{}", os.name);
    match command {
        "apply" | "dry-run" => {
            let path = args
                .get(1)
                .ok_or_else(|| format!("Usage: linux_post_install {command} <manifest>"))?;
            let path = PathBuf::from(path);
            let manifest = Manifest::read(&path)?;
            apply::apply(&manifest, &path, os.distro, &[], command == "dry-run")
        }
        "install" => {
            let path = Path::new(DEFAULT_MANIFEST_PATH);
            let manifest = Manifest::read(path)?;
            let profiles = parse_profiles(&args[1..]);
            apply::apply(&manifest, path, os.distro, &profiles, false)
        }
        "capture" => {
            let path = args
                .get(1)
                .ok_or("Usage: linux_post_install capture <manifest>")?;
            let manifest = capture::capture(os.distro)?;
            manifest.write(Path::new(path))?;
            println!("Captured system state in {path}");
            Ok(())
        }
        unknown => Err(format!("Unknown command: {unknown}. Run with --help.")),
    }
}

fn parse_profiles(arguments: &[String]) -> Vec<String> {
    arguments
        .iter()
        .flat_map(|argument| argument.split(','))
        .map(str::trim)
        .filter(|profile| !profile.is_empty())
        .map(str::to_owned)
        .collect()
}

fn print_help() {
    println!(
        "Linux system replication\n\n\
         Usage:\n  \
         linux_post_install dry-run <manifest>\n  \
         linux_post_install apply <manifest>\n  \
         linux_post_install install [profile[,profile] ...]\n  \
         linux_post_install capture <manifest>\n\n\
         The install command reads {DEFAULT_MANIFEST_PATH}; apply and dry-run use\n\
         the manifest's selected_profiles."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_comma_separated_and_individual_profiles() {
        let args = vec!["base,desktop".into(), "development".into()];
        assert_eq!(parse_profiles(&args), ["base", "desktop", "development"]);
    }
}
