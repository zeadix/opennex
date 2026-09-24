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

# 杀掉遗留/僵死的 vite(端口占用者),始终由本脚本启动一个新的。
# 旧行为"复用已在跑的 vite"有坑: 该 vite 可能属于上一次已退出的
# dev.sh(trap 杀过但没杀干净)或已僵死 —— Tauri 窗口会连上一个
# 提供旧代码的服务,前端改动全部"看起来没生效"。
for pid in $(ss -tlnpH "sport = :5183" 2>/dev/null | grep -oP 'pid=\K[0-9]+' | sort -u); do
  kill "$pid" 2>/dev/null || true
done
sleep 0.3
npm run dev -- --port 5183 --strictPort > /tmp/opennex-vite.log 2>&1 &
VITE_PID=$!
for _ in $(seq 1 30); do
  curl -s -o /dev/null http://localhost:5183 && break
  sleep 0.3
done
if ! curl -s -o /dev/null http://localhost:5183; then
  echo "ERROR: vite dev server failed to start; see /tmp/opennex-vite.log" >&2
  exit 1
fi
echo "vite dev server ready on :5183 (fresh)"

cd src-tauri
cargo run
