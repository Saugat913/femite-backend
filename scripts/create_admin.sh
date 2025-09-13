#!/usr/bin/env bash
set -euo pipefail

# This script wraps the create_admin Rust binary.
# Required env:
#   ADMIN_EMAIL
#   ADMIN_PASSWORD
# One of:
#   DATABASE_URL or TEST_DATABASE_URL
# Optional:
#   RUN_MIGRATIONS=1 to run migrations before creating the admin

if [[ -z "${ADMIN_EMAIL:-}" || -z "${ADMIN_PASSWORD:-}" ]]; then
  echo "Usage: ADMIN_EMAIL=user@example.com ADMIN_PASSWORD=secret [DATABASE_URL=...] [RUN_MIGRATIONS=1] ./scripts/create_admin.sh" >&2
  exit 2
fi

# Build the binary if it doesn't exist
if ! cargo --version >/dev/null 2>&1; then
  echo "cargo is required to build the create_admin binary" >&2
  exit 1
fi

# Build in release for speed if desired; debug is fine for local use
cargo build --bin create_admin >/dev/null

# Run with current environment
./target/debug/create_admin

