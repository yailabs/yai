#!/bin/sh
# Official Linux source-install launcher. Resolve the installed symlink so every
# invocation uses this checkout's current release, never a stale copied binary.
set -eu
launcher=$(readlink -f -- "$0")
repo_root=$(CDPATH= cd -- "$(dirname -- "$launcher")/../.." && pwd)
binary="$repo_root/studio/src-tauri/target/release/yai-studio"
if [ ! -x "$binary" ]; then
    printf '%s\n' "yai-studio: release binary missing: $binary" "Build from studio/: npm run desktop:build -- -- --locked" >&2
    exit 127
fi
export YAI_HOME="${YAI_HOME:-$HOME/.yai}"
# Case-sized governed inference may spend longer than the transport library's
# 300-second default in prefill. Keep one bounded deadline for the resident Host
# started by Studio; an explicit operator value still takes precedence.
export YAI_PROVIDER_RESPONSE_TIMEOUT_SECS="${YAI_PROVIDER_RESPONSE_TIMEOUT_SECS-3600}"
# Prefer the qualified X11 path when X/XWayland is available; retain explicit
# operator overrides and do not invent a display on a Wayland-only machine.
if [ -n "${DISPLAY:-}" ]; then
    export GDK_BACKEND="${GDK_BACKEND:-x11}"
fi
export WEBKIT_DISABLE_DMABUF_RENDERER="${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"
exec "$binary" "$@"
