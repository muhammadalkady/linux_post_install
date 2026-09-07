#!/usr/bin/env bash
# Installs Claude Code via the official native installer. Not a distro
# package, so it can't live in replica.toml's package profiles.
set -euo pipefail

if command -v claude >/dev/null 2>&1; then
    echo "Claude Code already installed ($(claude --version 2>/dev/null))"
    exit 0
fi

curl -fsSL https://claude.ai/install.sh | bash
