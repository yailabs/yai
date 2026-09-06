#!/bin/sh
set -eu

run_dir="build/tmp/new11/daemon-ipc-$$"
socket_path="$run_dir/yaid.sock"
log_path="$run_dir/yaid.log"

mkdir -p "$run_dir"
rm -f "$socket_path"

build/yaid --socket "$socket_path" --foreground >"$log_path" 2>&1 &
daemon_pid=$!

cleanup() {
  if kill -0 "$daemon_pid" 2>/dev/null; then
    kill "$daemon_pid" 2>/dev/null || true
    wait "$daemon_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

tries=0
while [ ! -S "$socket_path" ]; do
  tries=$((tries + 1))
  if [ "$tries" -gt 50 ]; then
    cat "$log_path" >&2 || true
    exit 1
  fi
  sleep 0.1
done

status_output=$(./yai daemon status --socket "$socket_path")
info_output=$(./yai daemon info --socket "$socket_path")
shutdown_output=$(./yai daemon shutdown --socket "$socket_path")

printf '%s\n' "$status_output" | grep '"status":"ok"' >/dev/null
printf '%s\n' "$status_output" | grep '"version":"' >/dev/null
printf '%s\n' "$info_output" | grep '"status":"ok"' >/dev/null
printf '%s\n' "$shutdown_output" | grep '"status":"ok"' >/dev/null

wait "$daemon_pid"
trap - EXIT INT TERM

printf 'daemon:started\n'
printf 'ipc:status ok\n'
printf 'ipc:info ok\n'
printf 'daemon:shutdown ok\n'
