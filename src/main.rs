mod os_details;
pub mod package;

use crate::os_details::get_os_details;
use crate::package::install_packages;

const PACKAGES: [&str; 3] = ["git", "curl", "wget"];

fn main() {
    let os_details = get_os_details();
    if let Some(os_details) = os_details {
        println!("{}", os_details.name);
        match install_packages(&os_details.distro.package_manager(), &PACKAGES) {
            Ok(status) => {
                if status.success() {
                    println!("Completed successfully.")
                } else {
                    println!("Aborted.")
                }
            }
            Err(error) => {
                eprintln!("{}.", error)
            }
        }
    }
}
