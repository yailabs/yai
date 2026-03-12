/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include <yai/sdk/client.h>
#include <yai/sdk/errors.h>
#include <yai/sdk/container.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Canonical source-plane aliases from container command taxonomy. */
#define YAI_SDK_SOURCE_CMD_ENROLL YAI_SDK_CMD_SOURCE_ENROLL
#define YAI_SDK_SOURCE_CMD_ATTACH YAI_SDK_CMD_SOURCE_ATTACH
#define YAI_SDK_SOURCE_CMD_EMIT YAI_SDK_CMD_SOURCE_EMIT
#define YAI_SDK_SOURCE_CMD_STATUS YAI_SDK_CMD_SOURCE_STATUS
#define YAI_SDK_SOURCE_QUERY_FAMILY YAI_SDK_CONTAINER_QUERY_FAMILY_SOURCE

typedef struct yai_sdk_source_mediation_state {
    int owner_canonical;
    int transport_ready;
    int network_gate_ready;
    int resource_gate_ready;
    int storage_gate_ready;
    char layer[16];
    char stage[64];
    char topology[96];
    char route[48];
} yai_sdk_source_mediation_state_t;

typedef struct yai_sdk_source_distribution_state {
    char source_enrollment_grant_id[128];
    char source_policy_snapshot_id[128];
    char source_capability_envelope_id[128];
    char policy_snapshot_version[64];
    char distribution_target_ref[256];
    char delegated_observation_scope[128];
    char delegated_mediation_scope[128];
    char delegated_enforcement_scope[128];
} yai_sdk_source_distribution_state_t;

typedef struct yai_sdk_source_enroll_request {
    const char *workspace_id;
    const char *source_label;
    const char *owner_ref;
    const char *source_node_id;
    const char *daemon_instance_id;
} yai_sdk_source_enroll_request_t;

typedef struct yai_sdk_source_enroll_reply {
    char workspace_id[64];
    char source_node_id[96];
    char daemon_instance_id[96];
    char owner_link_id[96];
    int registered;
    yai_sdk_source_distribution_state_t distribution;
    yai_sdk_source_mediation_state_t mediation;
} yai_sdk_source_enroll_reply_t;

typedef struct yai_sdk_source_attach_request {
    const char *workspace_id;
    const char *owner_workspace_id;
    const char *source_node_id;
    const char *binding_scope;
    const char *constraints_ref;
} yai_sdk_source_attach_request_t;

typedef struct yai_sdk_source_attach_reply {
    char workspace_id[64];
    char owner_workspace_id[64];
    char source_node_id[96];
    char source_binding_id[96];
    char attachment_status[32];
    yai_sdk_source_distribution_state_t distribution;
    yai_sdk_source_mediation_state_t mediation;
} yai_sdk_source_attach_reply_t;

typedef struct yai_sdk_source_asset {
    const char *source_asset_id;
    const char *source_binding_id;
    const char *locator;
    const char *asset_type;
    const char *provenance_fingerprint;
    const char *observation_state;
} yai_sdk_source_asset_t;

typedef struct yai_sdk_source_acquisition_event {
    const char *source_acquisition_event_id;
    const char *source_node_id;
    const char *source_binding_id;
    const char *source_asset_id;
    const char *event_type;
    int64_t observed_at_epoch;
    const char *idempotency_key;
    const char *delivery_status;
} yai_sdk_source_acquisition_event_t;

typedef struct yai_sdk_source_evidence_candidate {
    const char *source_evidence_candidate_id;
    const char *source_acquisition_event_id;
    const char *candidate_type;
    const char *derived_metadata_ref;
    const char *owner_resolution_status;
} yai_sdk_source_evidence_candidate_t;

typedef struct yai_sdk_source_emit_request {
    const char *workspace_id;
    const char *source_node_id;
    const char *source_binding_id;
    const char *idempotency_key;

    const yai_sdk_source_asset_t *assets;
    size_t assets_len;

    const yai_sdk_source_acquisition_event_t *events;
    size_t events_len;

    const yai_sdk_source_evidence_candidate_t *candidates;
    size_t candidates_len;
} yai_sdk_source_emit_request_t;

typedef struct yai_sdk_source_emit_reply {
    char workspace_id[64];
    char source_node_id[96];
    char source_binding_id[96];
    char idempotency_key[128];
    int accepted_assets;
    int accepted_events;
    int accepted_candidates;
    yai_sdk_source_mediation_state_t mediation;
} yai_sdk_source_emit_reply_t;

typedef struct yai_sdk_source_status_request {
    const char *workspace_id;
    const char *source_node_id;
    const char *daemon_instance_id;
    const char *health;
} yai_sdk_source_status_request_t;

typedef struct yai_sdk_source_status_reply {
    char workspace_id[64];
    char source_node_id[96];
    char daemon_instance_id[96];
    char health[48];
    yai_sdk_source_distribution_state_t distribution;
    yai_sdk_source_mediation_state_t mediation;
} yai_sdk_source_status_reply_t;

typedef struct yai_sdk_source_summary {
    int source_node_count;
    int source_daemon_instance_count;
    int source_binding_count;
    int source_asset_count;
    int source_acquisition_event_count;
    int source_evidence_candidate_count;
    int source_owner_link_count;
    int source_graph_node_count;
    int source_graph_edge_count;
} yai_sdk_source_summary_t;

void yai_sdk_source_mediation_state_init(yai_sdk_source_mediation_state_t *out);
void yai_sdk_source_distribution_state_init(yai_sdk_source_distribution_state_t *out);
void yai_sdk_source_enroll_reply_init(yai_sdk_source_enroll_reply_t *out);
void yai_sdk_source_attach_reply_init(yai_sdk_source_attach_reply_t *out);
void yai_sdk_source_emit_reply_init(yai_sdk_source_emit_reply_t *out);
void yai_sdk_source_status_reply_init(yai_sdk_source_status_reply_t *out);
void yai_sdk_source_summary_init(yai_sdk_source_summary_t *out);

/* Typed source-plane API surface (preferred over raw control-call JSON). */
int yai_sdk_source_enroll(
    yai_sdk_client_t *client,
    const yai_sdk_source_enroll_request_t *req,
    yai_sdk_reply_t *out);

int yai_sdk_source_attach(
    yai_sdk_client_t *client,
    const yai_sdk_source_attach_request_t *req,
    yai_sdk_reply_t *out);

int yai_sdk_source_emit(
    yai_sdk_client_t *client,
    const yai_sdk_source_emit_request_t *req,
    yai_sdk_reply_t *out);

int yai_sdk_source_status(
    yai_sdk_client_t *client,
    const yai_sdk_source_status_request_t *req,
    yai_sdk_reply_t *out);

/* Source read-surface baseline (query family fallback-backed). */
int yai_sdk_source_summary(yai_sdk_client_t *client, yai_sdk_reply_t *out);

/* Typed parse helpers from generic SDK replies. */
int yai_sdk_source_enroll_reply_from_sdk(
    const yai_sdk_reply_t *reply,
    yai_sdk_source_enroll_reply_t *out);

int yai_sdk_source_attach_reply_from_sdk(
    const yai_sdk_reply_t *reply,
    yai_sdk_source_attach_reply_t *out);

int yai_sdk_source_emit_reply_from_sdk(
    const yai_sdk_reply_t *reply,
    yai_sdk_source_emit_reply_t *out);

int yai_sdk_source_status_reply_from_sdk(
    const yai_sdk_reply_t *reply,
    yai_sdk_source_status_reply_t *out);

int yai_sdk_source_summary_from_sdk(
    const yai_sdk_reply_t *reply,
    yai_sdk_source_summary_t *out);

static inline int yai_sdk_source_is_query_family(const char *query_family)
{
    return query_family &&
           strcmp(query_family, YAI_SDK_SOURCE_QUERY_FAMILY) == 0;
}

#ifdef __cplusplus
}
#endif
