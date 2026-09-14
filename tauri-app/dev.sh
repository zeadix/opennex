#!/usr/bin/env bash
# One-shot dev launcher: starts the vite dev server in the background,
# runs the Tauri app (debug builds auto-connect to it for hot reload),
# and cleans the server up on exit.
set -e
cd "$(dirname "$0")"

cleanup() {
  [ -n "$VITE_PID" ] && kill "$VITE_PID" 2>/dev/null
}
trap cleanup EXIT

# Reuse a running vite if the port is already up; else start one.
if curl -s -o /dev/null http://localhost:5183; then
  echo "vite already running on :5183"
else
  npm run dev > /tmp/opennex-vite.log 2>&1 &
  VITE_PID=$!
  for _ in $(seq 1 30); do
    curl -s -o /dev/null http://localhost:5183 && break
    sleep 0.3
  done
  echo "vite dev server ready on :5183"
fi

cd src-tauri
cargo run
