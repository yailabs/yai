/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef enum yai_sdk_runtime_target_kind {
    YAI_SDK_RUNTIME_TARGET_UNKNOWN = 0,
    YAI_SDK_RUNTIME_TARGET_WORKSPACE_OWNER = 1,
    YAI_SDK_RUNTIME_TARGET_EDGE_RUNTIME = 2,
    YAI_SDK_RUNTIME_TARGET_MESH_PEER = 3,
    YAI_SDK_RUNTIME_TARGET_OVERLAY_REMOTE_OWNER = 4,
    YAI_SDK_RUNTIME_TARGET_OVERLAY_REMOTE_PEER = 5,
} yai_sdk_runtime_target_kind_t;

typedef enum yai_sdk_mesh_plane {
    YAI_SDK_MESH_PLANE_UNKNOWN = 0,
    YAI_SDK_MESH_PLANE_DISCOVERY = 1,
    YAI_SDK_MESH_PLANE_COORDINATION = 2,
    YAI_SDK_MESH_PLANE_AUTHORITY = 3,
} yai_sdk_mesh_plane_t;

typedef struct yai_sdk_runtime_target {
    yai_sdk_runtime_target_kind_t target_kind;
    yai_sdk_mesh_plane_t mesh_plane;
    char workspace_id[64];
    char owner_id[96];
    char node_id[96];
    char daemon_instance_id[96];
    char mesh_role[32];
    char locator_ref[512];
    char overlay_identity_ref[256];
    char ingress_ref[512];
} yai_sdk_runtime_target_t;

typedef struct yai_sdk_policy_distribution_descriptor {
    char source_enrollment_grant_id[128];
    char source_policy_snapshot_id[128];
    char source_capability_envelope_id[128];
    char distribution_target_ref[256];
    char delegated_observation_scope[128];
    char delegated_mediation_scope[128];
    char delegated_enforcement_scope[128];
    char validity_state[32];
    char freshness_state[32];
} yai_sdk_policy_distribution_descriptor_t;

typedef struct yai_sdk_remote_association_state {
    char reachability_state[32];
    char ingress_state[32];
    char transport_state[32];
    char overlay_state[32];
    char degradation_state[32];
    int restricted;
} yai_sdk_remote_association_state_t;

void yai_sdk_runtime_target_init(yai_sdk_runtime_target_t *out);
void yai_sdk_policy_distribution_descriptor_init(yai_sdk_policy_distribution_descriptor_t *out);
void yai_sdk_remote_association_state_init(yai_sdk_remote_association_state_t *out);

#ifdef __cplusplus
}
#endif
