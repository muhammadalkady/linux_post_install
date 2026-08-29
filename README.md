# Linux post-install

A small, manifest-driven Linux system replication tool. It detects the current
distribution, translates logical package profiles into native package-manager
commands, and can also bootstrap DankLinux, install Flatpaks, enable services,
and restore configuration files.

> [!CAUTION]
> Applying a manifest installs packages, runs an optional remote installer,
> enables services, and overwrites mapped dotfiles. Always run `dry-run` and
> review both the manifest and printed commands first.

## Requirements

- Linux on a [supported distribution](#supported-distributions)
- A current Rust toolchain (`cargo` and `rustc`)
- `sudo` access for native packages and system services
- Flatpak and systemd when the corresponding manifest sections are used

## Quick start

Build the program and preview the example without changing the system:

```sh
cargo run -- dry-run replica.example.toml
```

Copy the example, edit its profiles and paths, and apply it:

```sh
cp replica.example.toml replica.toml
cargo run -- dry-run replica.toml
cargo run -- apply replica.toml
```

To apply only selected profiles from the default `replica.toml`:

```sh
cargo run -- install base,desktop
```

Every command is printed before it runs. Package installation and system
services use `sudo`, so the system may request authentication.

## Commands

| Command | Purpose |
| --- | --- |
| `dry-run <manifest>` | Print all planned operations without changing the system |
| `apply <manifest>` | Apply the manifest's `selected_profiles` and other declared state |
| `install [profiles...]` | Apply named profiles from `replica.toml`; accepts comma-separated or space-separated names |
| `capture <manifest>` | Write detected native packages, Flatpaks, and enabled user services to a new manifest |

## Capture the current system

Create a starting manifest from explicitly installed native packages, Flatpak
applications, and enabled user services:

```sh
cargo run -- capture captured.toml
```

Capture deliberately does not copy personal files or infer which system
services should be enabled. Review the generated package list, split it into
useful profiles, then add selected dotfile mappings and services manually.

## Manifest structure

- `selected_profiles` controls which profiles `apply` uses.
- `[profiles.<name>]` defines packages shared across distributions.
- `[profiles.<name>.packages]` maps `os-release` IDs to native package names.
- `[danklinux]` controls the official `https://install.danklinux.com` bootstrap.
- `flatpaks`, `system_services`, and `user_services` restore application and
  service state.
- `[[dotfiles]]` copies a local file or directory to its target. Relative source
  paths are resolved from the manifest directory, and `~/` targets use the
  current user's home directory.

Dotfile copying overlays existing targets; it does not remove unrelated files.
Keep secrets, machine-specific credentials, browser profiles, and SSH keys out
of a shared dotfiles repository.

Set `template = true` on a dotfile mapping to replace `{{HOME}}` inside UTF-8
files during restoration. The workstation manifest uses this for DMS's session
file because DMS records the wallpaper as an absolute path.

The checked-in `replica.toml` is the local workstation profile. Its dotfile
bundle preserves the active DMS theme and layout, Hyprland animations and
keybindings, Ghostty shaders, the ML4W runtime helpers required by the Hyprland
configuration, and GTK/Kvantum application theming. Generated backups, caches,
plugin checkout metadata, personal GTK bookmarks, and the large ML4W wallpaper
collection are intentionally excluded.

The active Hyprland paper display effect is self-contained in
`dotfiles/hypr/papershell.lua` and `dotfiles/hypr/shaders/`. Press
`Super+Shift+G` to cycle between Paper Grain, Book Comfort, and no screen
shader. Ghostty's cursor shaders are stored separately under
`dotfiles/ghostty/shaders/`.

Only the currently selected wallpaper is bundled, as
`dotfiles/wallpapers/current.jpg`; the rest of the local wallpaper collection
is not copied.

## Supported distributions

The tool recognizes Arch Linux, Manjaro, EndeavourOS, Debian, Ubuntu, Linux
Mint, Pop!_OS, elementary OS, Kali, Fedora, RHEL, CentOS, Rocky Linux, AlmaLinux,
Amazon Linux, openSUSE/SLES, Alpine, Void Linux, Gentoo, and Solus. A manifest
may define distribution-specific package lists using the corresponding
`/etc/os-release` ID (for example, `fedora`, `arch`, or `ubuntu`).

Package capture output varies by package manager and should always be reviewed.
In particular, the generated `captured` profile is a starting point rather than
a guarantee that the manifest is portable to another distribution.

## Development

Run the test suite and compiler checks before submitting changes:

```sh
cargo test
cargo check
```
