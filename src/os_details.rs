use std::fs;

pub fn print_os_details() {
    let os_release = fs::read_to_string("/etc/os-release");
    match os_release {
        Ok(content) => {
            println!(
                "OS: {}, {}, {}",
                find_value(&content, "NAME").unwrap_or_default(),
                find_value(&content, "VERSION").unwrap_or_default(),
                find_value(&content, "RELEASE_TYPE").unwrap_or_default(),
            );
        }
        Err(error) => {
            eprintln!("Could not detect Linux distribution:\n{error}")
        }
    }
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

#[cfg(test)]
mod tests {
    use super::find_value;

    #[test]
    fn finds_and_unquotes_a_value() {
        let content = "NAME=\"Fedora Linux\"\nVERSION_ID=44\n";

        assert_eq!(find_value(content, "NAME"), Some("Fedora Linux"));
        assert_eq!(find_value(content, "VERSION_ID"), Some("44"));
    }

    #[test]
    fn skips_lines_without_a_separator() {
        let content = "invalid line\nNAME=Fedora Linux\n";

        assert_eq!(find_value(content, "NAME"), Some("Fedora Linux"));
    }

    #[test]
    fn returns_none_when_the_key_is_missing() {
        assert_eq!(find_value("NAME=Fedora Linux\n", "VERSION"), None);
    }
}
