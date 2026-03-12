/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum yai_sdk_liveness_state {
    YAI_SDK_LIVENESS_UNKNOWN = 0,
    YAI_SDK_LIVENESS_UP = 1,
    YAI_SDK_LIVENESS_DOWN = 2,
} yai_sdk_liveness_state_t;

typedef enum yai_sdk_binding_state {
    YAI_SDK_BINDING_UNKNOWN = 0,
    YAI_SDK_BINDING_NONE = 1,
    YAI_SDK_BINDING_SELECTED = 2,
    YAI_SDK_BINDING_ACTIVE = 3,
    YAI_SDK_BINDING_DEGRADED = 4,
    YAI_SDK_BINDING_UNBOUND = 5,
} yai_sdk_binding_state_t;

typedef enum yai_sdk_availability_state {
    YAI_SDK_AVAIL_UNKNOWN = 0,
    YAI_SDK_AVAIL_AVAILABLE = 1,
    YAI_SDK_AVAIL_UNAVAILABLE = 2,
    YAI_SDK_AVAIL_DEGRADED = 3,
} yai_sdk_availability_state_t;

typedef enum yai_sdk_recovery_state {
    YAI_SDK_RECOVERY_UNKNOWN = 0,
    YAI_SDK_RECOVERY_NONE = 1,
    YAI_SDK_RECOVERY_FRESH = 2,
    YAI_SDK_RECOVERY_RESTORED = 3,
    YAI_SDK_RECOVERY_RECOVERED = 4,
    YAI_SDK_RECOVERY_FAILED = 5,
} yai_sdk_recovery_state_t;

typedef struct yai_sdk_family_state {
    yai_sdk_availability_state_t availability;
    int ready;
    int bound;
    int degraded;
} yai_sdk_family_state_t;

typedef struct yai_sdk_runtime_state {
    yai_sdk_liveness_state_t liveness;
    yai_sdk_binding_state_t container_binding;
    yai_sdk_recovery_state_t recovery;

    int has_active_container;
    char container_id[64];
    char container_alias[64];

    yai_sdk_family_state_t exec;
    yai_sdk_family_state_t data;
    yai_sdk_family_state_t graph;
    yai_sdk_family_state_t cognition;

    char status[16];
    char code[64];
    char reason[128];
    char command_id[128];
    char target_plane[32];
} yai_sdk_runtime_state_t;

typedef struct yai_sdk_governance_state {
    char effect[32];
    char review_state[64];
    char authority_decision[32];
    int attachable;
    int blocked;
} yai_sdk_governance_state_t;

typedef struct yai_sdk_operational_summary {
    int source_node_count;
    int source_daemon_instance_count;
    int source_binding_count;
    int source_action_point_count;
    int source_policy_snapshot_count;
    int source_enrollment_grant_count;
    int source_capability_envelope_count;
    int source_acquisition_event_count;
    int source_ingest_outcome_count;
    int source_evidence_candidate_count;

    int mesh_coordination_membership_count;
    int mesh_peer_awareness_count;
    int mesh_peer_legitimacy_count;
    int mesh_authority_scope_count;

    int mesh_transport_endpoint_count;
    int mesh_transport_path_state_count;
    int mesh_owner_remote_ingress_count;
    int mesh_owner_remote_ingress_decision_count;
    int mesh_overlay_presence_count;
    int mesh_overlay_target_association_count;
    int mesh_overlay_path_mutation_count;
} yai_sdk_operational_summary_t;

typedef struct yai_sdk_control_call {
    const char *target_plane;
    const char *command_id;
    const char *const *argv;
    size_t argv_len;
} yai_sdk_control_call_t;

void yai_sdk_runtime_state_init(yai_sdk_runtime_state_t *out);
int yai_sdk_runtime_state_from_reply_json(const char *exec_reply_json, yai_sdk_runtime_state_t *out);

void yai_sdk_governance_state_init(yai_sdk_governance_state_t *out);
int yai_sdk_governance_state_from_reply_json(const char *exec_reply_json, yai_sdk_governance_state_t *out);

void yai_sdk_operational_summary_init(yai_sdk_operational_summary_t *out);
int yai_sdk_operational_summary_from_reply_json(const char *exec_reply_json,
                                                yai_sdk_operational_summary_t *out);

int yai_sdk_control_call_to_json(
    const yai_sdk_control_call_t *call,
    char *out_json,
    size_t out_cap);

#ifdef __cplusplus
}
#endif
