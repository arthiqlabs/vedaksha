#!/usr/bin/env bash
# One-time setup: activate repo-local git hooks for clean-room enforcement.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
git config core.hooksPath .githooks
chmod +x .githooks/*

echo "Hooks activated from .githooks/"
echo "Active hooks:"
ls -1 .githooks/ | sed 's/^/  /'
