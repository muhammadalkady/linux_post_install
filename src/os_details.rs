use std::fs;

#[derive(Debug)]
pub struct OSDetails {
    pub name: String,
    pub distro: Distro,
}

#[derive(Debug)]
pub enum Distro {
    Arch,
    Debian,
    Ubuntu,
    Fedora,
}

#[derive(Debug)]
pub enum PackageManager {
    Apt,
    Pacman,
    Dnf,
}

impl PackageManager {
    pub fn executable(&self) -> &str {
        match self {
            PackageManager::Apt => "apt",
            PackageManager::Pacman => "pacman",
            PackageManager::Dnf => "dnf",
        }
    }

    pub fn install_arg(&self) -> &str {
        match self {
            PackageManager::Dnf | PackageManager::Apt => "install",
            PackageManager::Pacman => "-S",
        }
    }
}

impl Distro {
    pub fn package_manager(&self) -> PackageManager {
        match self {
            Distro::Arch => PackageManager::Pacman,
            Distro::Debian | Distro::Ubuntu => PackageManager::Apt,
            Distro::Fedora => PackageManager::Dnf,
        }
    }
}

pub fn get_os_details() -> Option<OSDetails> {
    let os_release = fs::read_to_string("/etc/os-release");
    match os_release {
        Ok(content) => {
            let name = format!(
                "OS: {}, {}, {}",
                find_value(&content, "NAME").unwrap_or_default(),
                find_value(&content, "VERSION").unwrap_or_default(),
                find_value(&content, "RELEASE_TYPE").unwrap_or_default(),
            );
            let distro = match find_value(&content, "ID").unwrap_or_default() {
                "arch" => Some(Distro::Arch),
                "fedora" => Some(Distro::Fedora),
                "debian" => Some(Distro::Debian),
                "ubuntu" => Some(Distro::Ubuntu),
                _ => return None,
            };
            if distro.is_some() {
                return Some(OSDetails {
                    name,
                    distro: distro.unwrap(),
                });
            }
        }
        Err(error) => {
            eprintln!("Could not detect Linux distribution:\n{error}")
        }
    }
    None
}

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
