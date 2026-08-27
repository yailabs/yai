#!/usr/bin/env bash
set -euo pipefail

# YAI - NET boundary guard
#
# Purpose:
#   Keep NET as an integrated YAI runtime module, not a repository-shaped
#   subtree or authority/execution owner.
#
# Scope:
#   Checks NET integrated topology, public headers, work-owned spine/snapshots
#   and forbidden implementation signals.
#
# Non-goals:
#   Does not compile NET or validate runtime transport behavior.
#
require_file() {
  if [[ ! -f "$1" ]]; then
    printf 'missing NET boundary file: %s\n' "$1" >&2
    exit 1
  fi
}

require_dir() {
  if [[ ! -d "$1" ]]; then
    printf 'missing NET boundary dir: %s\n' "$1" >&2
    exit 1
  fi
}

forbidden_path() {
  if [[ -e "$1" ]]; then
    printf 'forbidden NET repository-shaped path: %s\n' "$1" >&2
    exit 1
  fi
}

require_phrase() {
  local corpus="$1"
  local phrase="$2"
  if [[ "$corpus" != *"$phrase"* ]]; then
    printf 'missing NET boundary phrase: %s\n' "$phrase" >&2
    exit 1
  fi
}

require_dir "net"
require_file "net/README.md"
require_file "proto/net.md"

for area in core registry stream node capability endpoint health lifecycle transport route observe compat; do
  require_dir "net/$area"
done

for path in \
  include/yai/net/net.h \
  include/yai/net/version.h \
  include/yai/net/error.h \
  include/yai/net/ids.h \
  include/yai/net/stream.h \
  include/yai/net/transport.h \
  include/yai/net/node.h \
  include/yai/net/capability.h \
  include/yai/net/endpoint.h \
  include/yai/net/health.h \
  include/yai/net/invocation.h \
  include/yai/net/lifecycle.h \
  include/yai/net/trace.h \
  include/yai/net/receipt.h \
  include/yai/net/metrics.h \
  proto/fixtures/net/stream/request.json \
  proto/fixtures/net/stream/complete.json \
  proto/fixtures/net/node/local-process.json \
  proto/fixtures/net/node/external-node.json \
  proto/fixtures/net/capability/neural-llm-decode.json \
  proto/fixtures/net/capability/generic-external.json \
  proto/fixtures/net/endpoint/local-process.json \
  proto/fixtures/net/endpoint/localhost-http.json \
  proto/fixtures/net/endpoint/future-transport.json \
  proto/fixtures/net/health/node-health.json \
  proto/fixtures/net/health/endpoint-liveness.json \
  proto/fixtures/net/health/capability-readiness.json \
  proto/fixtures/net/lifecycle/request-start.json \
  proto/fixtures/net/lifecycle/report-running.json \
  proto/fixtures/net/lifecycle/report-failed.json \
  proto/fixtures/net/transport/localhost-http.json \
  proto/fixtures/net/transport/local-ipc.json \
  proto/fixtures/net/transport/future-transport.json \
  proto/fixtures/net/ipc/abstract-local.json \
  proto/fixtures/net/ipc/unix-socket-future.json \
  proto/fixtures/net/ipc/memory-channel-future.json \
  proto/schemas/net-stream-envelope.v1.schema.json \
  proto/schemas/net-node-identity.v1.schema.json \
  proto/schemas/net-capability-advertisement.v1.schema.json \
  proto/schemas/net-endpoint-descriptor.v1.schema.json \
  proto/schemas/net-health-report.v1.schema.json \
  proto/schemas/net-lifecycle-request.v1.schema.json \
  proto/schemas/net-lifecycle-report.v1.schema.json \
  proto/schemas/net-transport-adapter.v1.schema.json \
  proto/schemas/net-local-ipc-channel.v1.schema.json \
  work/spines/net-spine.md; do
  require_file "$path"
done

if ! grep -Fx 'Reference version: NET.SPINE.9C.0' work/spines/net-spine.md >/dev/null; then
  printf 'work/spines/net-spine.md must declare Reference version: NET.SPINE.9C.0\n' >&2
  exit 1
fi

if ! find net -path 'net/*/*.c' -print -quit | grep -q .; then
  printf 'NET boundary requires integrated C translation units under net/<area>/\n' >&2
  exit 1
fi

for source in \
  net/core/net.c \
  net/stream/stream.c \
  net/transport/transport.c; do
  require_file "$source"
done

if ! grep -F '## Node Identity' proto/net.md >/dev/null; then
  printf 'proto/net.md must define Node Identity\n' >&2
  exit 1
fi

if ! grep -F '## Capability Advertisement' proto/net.md >/dev/null; then
  printf 'proto/net.md must define Capability Advertisement\n' >&2
  exit 1
fi

if ! grep -F '## Endpoint Descriptor' proto/net.md >/dev/null; then
  printf 'proto/net.md must define Endpoint Descriptor\n' >&2
  exit 1
fi

if ! grep -F '## Health, Readiness and Liveness' proto/net.md >/dev/null; then
  printf 'proto/net.md must define Health, Readiness and Liveness\n' >&2
  exit 1
fi

if ! grep -F '## Local Service Lifecycle' proto/net.md >/dev/null; then
  printf 'proto/net.md must define Local Service Lifecycle\n' >&2
  exit 1
fi

if ! grep -F '## Transport Adapter' proto/net.md >/dev/null; then
  printf 'proto/net.md must define Transport Adapter\n' >&2
  exit 1
fi

if ! grep -F '## Local IPC Channel' proto/net.md >/dev/null; then
  printf 'proto/net.md must define Local IPC Channel\n' >&2
  exit 1
fi

for path in \
  net/docs \
  net/src \
  net/tests \
  net/fixtures \
  net/benches \
  net/examples \
  net/Makefile \
  work/protocols \
  work/protocols/net.md \
  work/protocols/fixtures \
  work/protocols/schemas \
  clori \
  third_party/clori \
  vendor/clori \
  external/clori \
  docs/""spines/net-spine.md \
  docs/archive/engineering/waves/net-spine-0-root-component-scaffold-boundary-guard.md \
  docs/""engineering/net-yai-boundary.md \
  docs/""engineering/external-clori-boundary.md; do
  forbidden_path "$path"
done

corpus="$(cat \
  net/README.md \
  proto/net.md \
  work/spines/net-spine.md \
  work/archive/architecture-snapshots/net-root-substrate.md \
  work/archive/engineering-snapshots/net-yai-boundary.md \
  work/archive/engineering-snapshots/external-clori-boundary.md \
  | tr '\n' ' ')"

for phrase in \
  "YAI controls authority." \
  "NET moves streams." \
  "External nodes execute capabilities." \
  "CLORI executes neural computation."; do
  require_phrase "$corpus" "$phrase"
done

while IFS= read -r active_net_file; do
  if [[ "$active_net_file" == tools/checks/check-net-boundary.sh ]]; then
    continue
  fi
  if grep -n -E 'work/protocols/(net\.md|fixtures|schemas)' "$active_net_file" >/tmp/yai-net-old-protocol-refs.$$ 2>/dev/null; then
    if [[ "$active_net_file" == work/waves/* ]] &&
       grep -F 'Corrective note: NET.SPINE.3R moved protocol material' "$active_net_file" >/dev/null; then
      rm -f /tmp/yai-net-old-protocol-refs.$$
      continue
    fi
    cat /tmp/yai-net-old-protocol-refs.$$ >&2
    rm -f /tmp/yai-net-old-protocol-refs.$$
    printf 'NET protocol material must reference proto/, not work/protocols\n' >&2
    exit 1
  fi
  rm -f /tmp/yai-net-old-protocol-refs.$$
done < <(find net include/yai/net tools/checks proto work/spines work/waves -type f 2>/dev/null)

for forbidden in \
  "NET owns case authority" \
  "NET owns policy resolution" \
  "NET executes neural computation" \
  "NET loads models" \
  "NET mutates RuntimeGraph" \
  "NET writes YAI journal" \
  "NET writes YAI memory" \
  "CLORI is inside YAI" \
  "CLORI is vendored into YAI"; do
  if [[ "$corpus" == *"$forbidden"* ]]; then
    printf 'forbidden NET boundary claim: %s\n' "$forbidden" >&2
    exit 1
  fi
done

if grep -R -n -E '#[[:space:]]*include[[:space:]]+[<"]((system|engine)/|yai/(system|engine)/)' net include/yai/net >/tmp/yai-net-boundary-includes.$$ 2>/dev/null; then
  if ! grep -F 'NET_INTERNAL_INCLUDE_ALLOWED' /tmp/yai-net-boundary-includes.$$ >/dev/null; then
    cat /tmp/yai-net-boundary-includes.$$ >&2
    rm -f /tmp/yai-net-boundary-includes.$$
    printf 'NET.SPINE.0R must not include system/ or engine/ internals\n' >&2
    exit 1
  fi
fi
rm -f /tmp/yai-net-boundary-includes.$$

if grep -R -n -E '\b(socket|connect|listen|accept|bind|getaddrinfo|recv|send|read|write|close|open|mkfifo|pipe)[[:space:]]*\(' net include/yai/net >/tmp/yai-net-boundary-network.$$ 2>/dev/null; then
  cat /tmp/yai-net-boundary-network.$$ >&2
  rm -f /tmp/yai-net-boundary-network.$$
  printf 'NET.SPINE.0R must not contain network implementation symbols\n' >&2
  exit 1
fi
rm -f /tmp/yai-net-boundary-network.$$

if grep -R -n -i 'HTTP server implementation' net include/yai/net >/tmp/yai-net-boundary-http.$$ 2>/dev/null; then
  cat /tmp/yai-net-boundary-http.$$ >&2
  rm -f /tmp/yai-net-boundary-http.$$
  printf 'NET.SPINE.0R must not claim HTTP server implementation\n' >&2
  exit 1
fi
rm -f /tmp/yai-net-boundary-http.$$

if grep -R -n -i -E '\b(curl|http request)\b' net include/yai/net >/tmp/yai-net-boundary-http-request.$$ 2>/dev/null; then
  cat /tmp/yai-net-boundary-http-request.$$ >&2
  rm -f /tmp/yai-net-boundary-http-request.$$
  printf 'NET.SPINE.8 must not contain HTTP request implementation signals\n' >&2
  exit 1
fi
rm -f /tmp/yai-net-boundary-http-request.$$

if grep -R -n -i -E '\b(mdns|dns_sd|broadcast)\b' net --include='*.c' >/tmp/yai-net-boundary-discovery.$$ 2>/dev/null; then
  cat /tmp/yai-net-boundary-discovery.$$ >&2
  rm -f /tmp/yai-net-boundary-discovery.$$
  printf 'NET.SPINE.9C must not contain discovery implementation signals\n' >&2
  exit 1
fi
rm -f /tmp/yai-net-boundary-discovery.$$

if grep -R -n -E '\b(fork|exec|system|popen|kill|waitpid)[[:space:]]*\(' net include/yai/net >/tmp/yai-net-boundary-process.$$ 2>/dev/null; then
  cat /tmp/yai-net-boundary-process.$$ >&2
  rm -f /tmp/yai-net-boundary-process.$$
  printf 'NET.SPINE.7 must not contain process management implementation symbols\n' >&2
  exit 1
fi
rm -f /tmp/yai-net-boundary-process.$$

if grep -R -n -E '\b(launchctl|systemctl|service start|service stop)\b' net include/yai/net >/tmp/yai-net-boundary-service.$$ 2>/dev/null; then
  cat /tmp/yai-net-boundary-service.$$ >&2
  rm -f /tmp/yai-net-boundary-service.$$
  printf 'NET.SPINE.7 must not contain service manager implementation symbols\n' >&2
  exit 1
fi
rm -f /tmp/yai-net-boundary-service.$$

printf 'check-net-boundary: ok\n'
