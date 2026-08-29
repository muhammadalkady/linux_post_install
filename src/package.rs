use crate::os_details::PackageManager;
use std::io;
use std::process::{Command, ExitStatus};

pub fn install_packages(
    package_manager: &PackageManager,
    packages_names: &[&str],
) -> io::Result<ExitStatus> {
    let mut command = Command::new("sudo");
    command.arg(package_manager.executable());
    command.args(package_manager.install_args());
    command.args(packages_names);
    command.status()
}
