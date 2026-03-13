/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/sdk/models.h>
#include <yai/sdk/transport.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * Compatibility low-level client API.
 * Canonical runtime taxonomy entry is <yai/sdk/runtime.h>.
 */

/**
 * @brief Opaque SDK client handle.
 *
 * A client handle is not thread-safe for concurrent mutation.
 * Use one handle per thread or external synchronization.
 */
typedef struct yai_sdk_client yai_sdk_client_t;

/**
 * @brief Client open options.
 */
typedef struct yai_sdk_client_opts {
    /** Container identifier used by default for SDK runtime calls. */
    const char *container_id;
    /** Optional custom runtime ingress UDS path override. */
    const char *uds_path;
    /** Optional typed runtime endpoint target (preferred over raw uds_path when set). */
    const yai_sdk_runtime_endpoint_t *runtime_endpoint;
    /** Optional owner endpoint ref (for example uds:///tmp/yai-owner.sock). */
    const char *owner_endpoint_ref;
    /** Optional connect timeout placeholder (ms) for future transport backends. */
    int connect_timeout_ms;
    /** Optional connect attempts placeholder (>=1). */
    int connect_attempts;
    /** Authority arming flag (0/1). */
    int arming;
    /** Authority role string (e.g. "operator"). */
    const char *role;
    /** If non-zero, perform handshake lazily before first call. */
    int auto_handshake;
    /** Optional correlation id prefix used for trace generation. */
    const char *correlation_id;
} yai_sdk_client_opts_t;

typedef struct yai_sdk_reply {
    char *exec_reply_json;
    char status[8];
    char code[64];
    char reason[256];
    char summary[256];
    char hints[2][256];
    int hint_count;
    char details[512];
    char command_id[128];
    char trace_id[128];
    char target_plane[16];
} yai_sdk_reply_t;

int yai_sdk_client_open(yai_sdk_client_t **out, const yai_sdk_client_opts_t *opts);
void yai_sdk_client_close(yai_sdk_client_t *c);

int yai_sdk_client_set_authority(yai_sdk_client_t *c, int arming, const char *role);
int yai_sdk_client_set_container(yai_sdk_client_t *c, const char *container_id);
int yai_sdk_client_set_correlation_id(yai_sdk_client_t *c, const char *correlation_id);
int yai_sdk_client_handshake(yai_sdk_client_t *c);

/* Runtime endpoint/locator visibility for diagnostics and operator tooling. */
int yai_sdk_client_runtime_locator_state(
    const yai_sdk_client_t *c,
    yai_sdk_runtime_locator_state_t *out);

/* Raw control-call JSON path (compatibility). */
int yai_sdk_client_call_json(yai_sdk_client_t *c, const char *control_call_json, yai_sdk_reply_t *out);

/* Canonical SDK-2 request-model path. */
int yai_sdk_client_call(yai_sdk_client_t *c, const yai_sdk_control_call_t *call, yai_sdk_reply_t *out);

int yai_sdk_client_ping(yai_sdk_client_t *c, const char *command_id, yai_sdk_reply_t *out);

/* Extract canonical SDK-2 runtime/governance models from reply payloads. */
int yai_sdk_reply_runtime_state(const yai_sdk_reply_t *reply, yai_sdk_runtime_state_t *out);
int yai_sdk_reply_governance_state(const yai_sdk_reply_t *reply, yai_sdk_governance_state_t *out);

void yai_sdk_reply_free(yai_sdk_reply_t *r);

#ifdef __cplusplus
}
#endif
