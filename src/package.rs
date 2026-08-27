use crate::os_details::PackageManager;
use std::io;
use std::process::{Command, ExitStatus};

pub fn install_packages(
    package_manager: &PackageManager,
    packages_names: &[&str],
) -> io::Result<ExitStatus> {
    let mut command = Command::new("sudo");
    command.arg(package_manager.executable());
    command.arg(package_manager.install_arg());
    command.args(packages_names);
    command.status()
}
