#!/usr/bin/env bash
# Installs the custom systemd units tracked in dotfiles/systemd/ into
# /etc/systemd/system, then reloads and enables each one.
#
# The main `linux_post_install` tool copies dotfiles as the current user and
# only runs `systemctl enable` (not file installs) with sudo, so it cannot
# place unit files under /etc itself. Run this script once per machine
# instead, after cloning the repo.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
units_dir="$repo_root/dotfiles/systemd"

if [[ ! -d "$units_dir" ]]; then
    echo "No units directory at $units_dir" >&2
    exit 1
fi

shopt -s nullglob
units=("$units_dir"/*.service)
if [[ ${#units[@]} -eq 0 ]]; then
    echo "No .service files found in $units_dir" >&2
    exit 0
fi

for unit in "${units[@]}"; do
    name="$(basename "$unit")"
    echo "Installing $name"
    sudo cp "$unit" "/etc/systemd/system/$name"
done

sudo systemctl daemon-reload

for unit in "${units[@]}"; do
    name="$(basename "$unit")"
    echo "Enabling $name"
    sudo systemctl enable --now "$name"
done
