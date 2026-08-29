mod os_details;
pub mod package;

use crate::os_details::get_os_details;
use crate::package::install_packages;
use std::process::ExitCode;

const OS_RELEASE_PATH: &str = "/etc/os-release";
const PACKAGES: [&str; 3] = ["git", "curl", "wget"];

fn main() -> ExitCode {
    let os_details = get_os_details(OS_RELEASE_PATH);
    match os_details {
        Ok(os_details) => {
            println!("{}", os_details.name);
            match install_packages(&os_details.distro.package_manager(), &PACKAGES) {
                Ok(status) => {
                    if status.success() {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::FAILURE
                    }
                }
                Err(_) => ExitCode::FAILURE,
            }
        }
        Err(_) => ExitCode::FAILURE,
    }
}
