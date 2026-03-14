#pragma once

#include <yai/api/status.h>
#include <yai/data/query.h>
#include <yai/graph/summary.h>

typedef enum yai_runtime_mode
{
  YAI_RUNTIME_MODE_UNSPECIFIED = 0,
  YAI_RUNTIME_MODE_RUNTIME,
  YAI_RUNTIME_MODE_DIAGNOSTIC
} yai_runtime_mode_t;

/* Canonical runtime/operator entrypoint for this repository. */
#define YAI_BIN_MAIN "yai"

/* Canonical runtime ingress socket relative to $HOME. */
#define YAI_RUNTIME_INGRESS_SOCKET_REL ".yai/run/control.sock"
/* Canonical peer ingress socket relative to $HOME. */
#define YAI_RUNTIME_PEER_INGRESS_SOCKET_REL ".yai/run/peer.sock"

/* Canonical runtime pidfile relative to $HOME. */
#define YAI_RUNTIME_PIDFILE_REL ".yai/run/runtime.pid"

/* Optional absolute override for runtime ingress resolution. */
#define YAI_RUNTIME_INGRESS_ENV "YAI_RUNTIME_INGRESS"
/* Optional absolute override for peer ingress resolution. */
#define YAI_RUNTIME_PEER_INGRESS_ENV "YAI_RUNTIME_PEER_INGRESS"

/* Optional absolute override for runtime pidfile resolution. */
#define YAI_RUNTIME_PIDFILE_ENV "YAI_RUNTIME_PIDFILE"

/*
 * Secure peering baseline gate (NP-4/MT-3 readiness).
 * If REQUIRED is truthy and READY is not truthy, runtime startup must fail
 * before exposing peer ingress.
 */
#define YAI_SECURE_PEERING_REQUIRED_ENV "YAI_SECURE_PEERING_REQUIRED"
#define YAI_SECURE_PEERING_READY_ENV "YAI_SECURE_PEERING_READY"
