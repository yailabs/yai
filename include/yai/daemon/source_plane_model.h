#pragma once

#include <stddef.h>
#include <stdint.h>

#define YAI_SOURCE_NODE_ID_MAX 96
#define YAI_SOURCE_DAEMON_INSTANCE_ID_MAX 96
#define YAI_SOURCE_BINDING_ID_MAX 96
#define YAI_SOURCE_ASSET_ID_MAX 128
#define YAI_SOURCE_ACQUISITION_EVENT_ID_MAX 128
#define YAI_SOURCE_EVIDENCE_CANDIDATE_ID_MAX 128
#define YAI_SOURCE_ACTION_POINT_ID_MAX 128
#define YAI_SOURCE_OWNER_LINK_ID_MAX 96
#define YAI_SOURCE_ENROLLMENT_GRANT_ID_MAX 128
#define YAI_SOURCE_POLICY_SNAPSHOT_ID_MAX 128
#define YAI_SOURCE_CAPABILITY_ENVELOPE_ID_MAX 128
#define YAI_SOURCE_WORKSPACE_PEER_MEMBERSHIP_ID_MAX 128
#define YAI_SOURCE_WORKSPACE_ID_MAX 64
#define YAI_SOURCE_LABEL_MAX 128
#define YAI_SOURCE_REF_MAX 256
#define YAI_SOURCE_STATUS_MAX 32
#define YAI_SOURCE_KIND_MAX 64
#define YAI_SOURCE_HASH_MAX 96

#define YAI_SOURCE_RECORD_CLASS_NODE "source_node"
#define YAI_SOURCE_RECORD_CLASS_DAEMON_INSTANCE "source_daemon_instance"
#define YAI_SOURCE_RECORD_CLASS_BINDING "source_binding"
#define YAI_SOURCE_RECORD_CLASS_ASSET "source_asset"
#define YAI_SOURCE_RECORD_CLASS_ACQUISITION_EVENT "source_acquisition_event"
#define YAI_SOURCE_RECORD_CLASS_EVIDENCE_CANDIDATE "source_evidence_candidate"
#define YAI_SOURCE_RECORD_CLASS_ACTION_POINT "source_action_point"
#define YAI_SOURCE_RECORD_CLASS_OWNER_LINK "source_owner_link"
#define YAI_SOURCE_RECORD_CLASS_ENROLLMENT_GRANT "source_enrollment_grant"
#define YAI_SOURCE_RECORD_CLASS_POLICY_SNAPSHOT "source_policy_snapshot"
#define YAI_SOURCE_RECORD_CLASS_CAPABILITY_ENVELOPE "source_capability_envelope"
#define YAI_SOURCE_RECORD_CLASS_WORKSPACE_PEER_MEMBERSHIP "workspace_peer_membership"
#define YAI_SOURCE_RECORD_CLASS_INGEST_OUTCOME "source_ingest_outcome"
#define YAI_SOURCE_RECORD_CLASS_MESH_NODE "mesh_node"
#define YAI_SOURCE_RECORD_CLASS_MESH_DISCOVERY_ADVERTISEMENT "mesh_discovery_advertisement"
#define YAI_SOURCE_RECORD_CLASS_MESH_BOOTSTRAP_DESCRIPTOR "mesh_bootstrap_descriptor"
#define YAI_SOURCE_RECORD_CLASS_MESH_COORDINATION_MEMBERSHIP "mesh_coordination_membership"
#define YAI_SOURCE_RECORD_CLASS_MESH_PEER_AWARENESS "mesh_peer_awareness"
#define YAI_SOURCE_RECORD_CLASS_MESH_PEER_LEGITIMACY "mesh_peer_legitimacy"
#define YAI_SOURCE_RECORD_CLASS_MESH_AUTHORITY_SCOPE "mesh_authority_scope"
#define YAI_SOURCE_RECORD_CLASS_MESH_TRANSPORT_ENDPOINT "mesh_transport_endpoint"
#define YAI_SOURCE_RECORD_CLASS_MESH_TRANSPORT_PATH_STATE "mesh_transport_path_state"
#define YAI_SOURCE_RECORD_CLASS_MESH_TRANSPORT_CHANNEL_STATE "mesh_transport_channel_state"

typedef enum yai_source_contract_operation {
  YAI_SOURCE_CONTRACT_INVALID = 0,
  YAI_SOURCE_CONTRACT_ENROLL = 1,
  YAI_SOURCE_CONTRACT_ATTACH = 2,
  YAI_SOURCE_CONTRACT_EMIT = 3,
  YAI_SOURCE_CONTRACT_STATUS = 4
} yai_source_contract_operation_t;

typedef struct yai_source_node {
  char source_node_id[YAI_SOURCE_NODE_ID_MAX];
  char source_label[YAI_SOURCE_LABEL_MAX];
  char trust_level[YAI_SOURCE_STATUS_MAX];
  char owner_ref[YAI_SOURCE_REF_MAX];
  char state[YAI_SOURCE_STATUS_MAX];
} yai_source_node_t;

typedef struct yai_source_daemon_instance {
  char daemon_instance_id[YAI_SOURCE_DAEMON_INSTANCE_ID_MAX];
  char source_node_id[YAI_SOURCE_NODE_ID_MAX];
  char session_marker[YAI_SOURCE_REF_MAX];
  int64_t started_at_epoch;
  char health[YAI_SOURCE_STATUS_MAX];
  char build_marker[YAI_SOURCE_REF_MAX];
} yai_source_daemon_instance_t;

typedef struct yai_source_binding {
  char source_binding_id[YAI_SOURCE_BINDING_ID_MAX];
  char source_node_id[YAI_SOURCE_NODE_ID_MAX];
  char owner_workspace_id[YAI_SOURCE_WORKSPACE_ID_MAX];
  char binding_scope[YAI_SOURCE_KIND_MAX];
  char attachment_status[YAI_SOURCE_STATUS_MAX];
  char constraints_ref[YAI_SOURCE_REF_MAX];
} yai_source_binding_t;

typedef struct yai_source_asset {
  char source_asset_id[YAI_SOURCE_ASSET_ID_MAX];
  char source_binding_id[YAI_SOURCE_BINDING_ID_MAX];
  char locator[YAI_SOURCE_REF_MAX];
  char asset_type[YAI_SOURCE_KIND_MAX];
  char provenance_fingerprint[YAI_SOURCE_HASH_MAX];
  char observation_state[YAI_SOURCE_STATUS_MAX];
} yai_source_asset_t;

typedef struct yai_source_acquisition_event {
  char source_acquisition_event_id[YAI_SOURCE_ACQUISITION_EVENT_ID_MAX];
  char source_node_id[YAI_SOURCE_NODE_ID_MAX];
  char source_binding_id[YAI_SOURCE_BINDING_ID_MAX];
  char source_asset_id[YAI_SOURCE_ASSET_ID_MAX];
  char event_type[YAI_SOURCE_KIND_MAX];
  int64_t observed_at_epoch;
  char idempotency_key[YAI_SOURCE_HASH_MAX];
  char delivery_status[YAI_SOURCE_STATUS_MAX];
} yai_source_acquisition_event_t;

typedef struct yai_source_evidence_candidate {
  char source_evidence_candidate_id[YAI_SOURCE_EVIDENCE_CANDIDATE_ID_MAX];
  char source_acquisition_event_id[YAI_SOURCE_ACQUISITION_EVENT_ID_MAX];
  char candidate_type[YAI_SOURCE_KIND_MAX];
  char derived_metadata_ref[YAI_SOURCE_REF_MAX];
  char owner_resolution_status[YAI_SOURCE_STATUS_MAX];
} yai_source_evidence_candidate_t;

typedef struct yai_source_action_point {
  char source_action_point_id[YAI_SOURCE_ACTION_POINT_ID_MAX];
  char source_node_id[YAI_SOURCE_NODE_ID_MAX];
  char source_binding_id[YAI_SOURCE_BINDING_ID_MAX];
  char action_kind[YAI_SOURCE_KIND_MAX];
  char action_ref[YAI_SOURCE_REF_MAX];
  char mediation_scope[YAI_SOURCE_REF_MAX];
  char enforcement_scope[YAI_SOURCE_REF_MAX];
  char controllability_state[YAI_SOURCE_STATUS_MAX];
  int64_t updated_at_epoch;
} yai_source_action_point_t;

typedef struct yai_source_owner_link {
  char source_owner_link_id[YAI_SOURCE_OWNER_LINK_ID_MAX];
  char source_node_id[YAI_SOURCE_NODE_ID_MAX];
  char owner_ref[YAI_SOURCE_REF_MAX];
  char registration_status[YAI_SOURCE_STATUS_MAX];
  int64_t registered_at_epoch;
} yai_source_owner_link_t;

typedef struct yai_source_enrollment_grant {
  char source_enrollment_grant_id[YAI_SOURCE_ENROLLMENT_GRANT_ID_MAX];
  char source_node_id[YAI_SOURCE_NODE_ID_MAX];
  char daemon_instance_id[YAI_SOURCE_DAEMON_INSTANCE_ID_MAX];
  char owner_ref[YAI_SOURCE_REF_MAX];
  char enrollment_decision[YAI_SOURCE_STATUS_MAX];
  char trust_artifact_id[YAI_SOURCE_REF_MAX];
  char trust_artifact_token[YAI_SOURCE_HASH_MAX];
  char validity_state[YAI_SOURCE_STATUS_MAX];
  int64_t valid_from_epoch;
  int64_t refresh_after_epoch;
  int64_t expires_at_epoch;
  int revoked;
  int64_t issued_at_epoch;
} yai_source_enrollment_grant_t;

typedef struct yai_source_policy_snapshot {
  char source_policy_snapshot_id[YAI_SOURCE_POLICY_SNAPSHOT_ID_MAX];
  char source_node_id[YAI_SOURCE_NODE_ID_MAX];
  char daemon_instance_id[YAI_SOURCE_DAEMON_INSTANCE_ID_MAX];
  char owner_workspace_id[YAI_SOURCE_WORKSPACE_ID_MAX];
  char source_enrollment_grant_id[YAI_SOURCE_ENROLLMENT_GRANT_ID_MAX];
  char snapshot_version[YAI_SOURCE_KIND_MAX];
  char distribution_target_ref[YAI_SOURCE_REF_MAX];
  char validity_state[YAI_SOURCE_STATUS_MAX];
  int64_t valid_from_epoch;
  int64_t refresh_after_epoch;
  int64_t expires_at_epoch;
  int revoked;
  int64_t issued_at_epoch;
} yai_source_policy_snapshot_t;

typedef struct yai_source_capability_envelope {
  char source_capability_envelope_id[YAI_SOURCE_CAPABILITY_ENVELOPE_ID_MAX];
  char source_node_id[YAI_SOURCE_NODE_ID_MAX];
  char daemon_instance_id[YAI_SOURCE_DAEMON_INSTANCE_ID_MAX];
  char source_binding_id[YAI_SOURCE_BINDING_ID_MAX];
  char owner_workspace_id[YAI_SOURCE_WORKSPACE_ID_MAX];
  char source_enrollment_grant_id[YAI_SOURCE_ENROLLMENT_GRANT_ID_MAX];
  char observation_scope[YAI_SOURCE_REF_MAX];
  char mediation_scope[YAI_SOURCE_REF_MAX];
  char enforcement_scope[YAI_SOURCE_REF_MAX];
  char distribution_target_ref[YAI_SOURCE_REF_MAX];
  char validity_state[YAI_SOURCE_STATUS_MAX];
  int64_t valid_from_epoch;
  int64_t refresh_after_epoch;
  int64_t expires_at_epoch;
  int revoked;
  int64_t issued_at_epoch;
} yai_source_capability_envelope_t;

typedef struct yai_source_workspace_peer_membership {
  char workspace_peer_membership_id[YAI_SOURCE_WORKSPACE_PEER_MEMBERSHIP_ID_MAX];
  char owner_workspace_id[YAI_SOURCE_WORKSPACE_ID_MAX];
  char source_node_id[YAI_SOURCE_NODE_ID_MAX];
  char source_binding_id[YAI_SOURCE_BINDING_ID_MAX];
  char daemon_instance_id[YAI_SOURCE_DAEMON_INSTANCE_ID_MAX];
  char peer_role[YAI_SOURCE_KIND_MAX];
  char peer_scope[YAI_SOURCE_REF_MAX];
  char peer_state[YAI_SOURCE_STATUS_MAX];
  int64_t backlog_queued;
  int64_t backlog_retry_due;
  int64_t backlog_failed;
  char coverage_ref[YAI_SOURCE_REF_MAX];
  char overlap_state[YAI_SOURCE_STATUS_MAX];
  int64_t updated_at_epoch;
} yai_source_workspace_peer_membership_t;

typedef struct yai_source_mesh_node {
  char mesh_node_id[YAI_SOURCE_NODE_ID_MAX];
  char node_role[YAI_SOURCE_KIND_MAX];
  char mesh_id[YAI_SOURCE_REF_MAX];
  char owner_id[YAI_SOURCE_REF_MAX];
  char protocol_version[YAI_SOURCE_STATUS_MAX];
  char capabilities_ref[YAI_SOURCE_REF_MAX];
  char reachability_state[YAI_SOURCE_STATUS_MAX];
  char discovery_endpoint_ref[YAI_SOURCE_REF_MAX];
  char enrollment_mode[YAI_SOURCE_STATUS_MAX];
  char workspace_visibility_scope[YAI_SOURCE_REF_MAX];
  char discovery_state[YAI_SOURCE_STATUS_MAX];
  int64_t advertised_at_epoch;
} yai_source_mesh_node_t;

typedef struct yai_source_mesh_discovery_advertisement {
  char mesh_discovery_advertisement_id[YAI_SOURCE_REF_MAX];
  char mesh_node_id[YAI_SOURCE_NODE_ID_MAX];
  char node_role[YAI_SOURCE_KIND_MAX];
  char mesh_id[YAI_SOURCE_REF_MAX];
  char owner_id[YAI_SOURCE_REF_MAX];
  char protocol_version[YAI_SOURCE_STATUS_MAX];
  char capabilities_ref[YAI_SOURCE_REF_MAX];
  char reachability_state[YAI_SOURCE_STATUS_MAX];
  char discovery_endpoint_ref[YAI_SOURCE_REF_MAX];
  char enrollment_mode[YAI_SOURCE_STATUS_MAX];
  char workspace_visibility_scope[YAI_SOURCE_REF_MAX];
  char advertisement_status[YAI_SOURCE_STATUS_MAX];
  int64_t advertised_at_epoch;
} yai_source_mesh_discovery_advertisement_t;

typedef struct yai_source_mesh_bootstrap_descriptor {
  char mesh_bootstrap_descriptor_id[YAI_SOURCE_REF_MAX];
  char mesh_id[YAI_SOURCE_REF_MAX];
  char owner_id[YAI_SOURCE_REF_MAX];
  char owner_discovery_ref[YAI_SOURCE_REF_MAX];
  char peer_discovery_ref[YAI_SOURCE_REF_MAX];
  char bootstrap_seed_ref[YAI_SOURCE_REF_MAX];
  char visibility_scope[YAI_SOURCE_REF_MAX];
  char bootstrap_state[YAI_SOURCE_STATUS_MAX];
  int64_t refreshed_at_epoch;
} yai_source_mesh_bootstrap_descriptor_t;

typedef struct yai_source_mesh_coordination_membership {
  char mesh_coordination_membership_id[YAI_SOURCE_REF_MAX];
  char owner_workspace_id[YAI_SOURCE_WORKSPACE_ID_MAX];
  char mesh_id[YAI_SOURCE_REF_MAX];
  char mesh_node_id[YAI_SOURCE_NODE_ID_MAX];
  char node_role[YAI_SOURCE_KIND_MAX];
  char membership_state[YAI_SOURCE_STATUS_MAX];
  char awareness_scope[YAI_SOURCE_REF_MAX];
  char coverage_ref[YAI_SOURCE_REF_MAX];
  char overlap_state[YAI_SOURCE_STATUS_MAX];
  char scheduling_state[YAI_SOURCE_STATUS_MAX];
  char ordering_state[YAI_SOURCE_STATUS_MAX];
  char replay_state[YAI_SOURCE_STATUS_MAX];
  char conflict_state[YAI_SOURCE_STATUS_MAX];
  int64_t freshness_epoch;
  int64_t updated_at_epoch;
} yai_source_mesh_coordination_membership_t;

typedef struct yai_source_mesh_peer_awareness {
  char mesh_peer_awareness_id[YAI_SOURCE_REF_MAX];
  char owner_workspace_id[YAI_SOURCE_WORKSPACE_ID_MAX];
  char source_mesh_node_id[YAI_SOURCE_NODE_ID_MAX];
  char observed_mesh_node_id[YAI_SOURCE_NODE_ID_MAX];
  char observed_role[YAI_SOURCE_KIND_MAX];
  char observed_peer_state[YAI_SOURCE_STATUS_MAX];
  char observed_freshness[YAI_SOURCE_STATUS_MAX];
  char observed_coverage_ref[YAI_SOURCE_REF_MAX];
  char observed_overlap_state[YAI_SOURCE_STATUS_MAX];
  char coordination_hint_ref[YAI_SOURCE_REF_MAX];
  int64_t observed_at_epoch;
} yai_source_mesh_peer_awareness_t;

typedef struct yai_source_mesh_peer_legitimacy {
  char mesh_peer_legitimacy_id[YAI_SOURCE_REF_MAX];
  char owner_workspace_id[YAI_SOURCE_WORKSPACE_ID_MAX];
  char mesh_id[YAI_SOURCE_REF_MAX];
  char mesh_node_id[YAI_SOURCE_NODE_ID_MAX];
  char enrollment_state[YAI_SOURCE_STATUS_MAX];
  char trust_state[YAI_SOURCE_STATUS_MAX];
  char legitimacy_state[YAI_SOURCE_STATUS_MAX];
  char legitimacy_reason[YAI_SOURCE_REF_MAX];
  int64_t evaluated_at_epoch;
} yai_source_mesh_peer_legitimacy_t;

typedef struct yai_source_mesh_authority_scope {
  char mesh_authority_scope_id[YAI_SOURCE_REF_MAX];
  char owner_workspace_id[YAI_SOURCE_WORKSPACE_ID_MAX];
  char mesh_id[YAI_SOURCE_REF_MAX];
  char mesh_node_id[YAI_SOURCE_NODE_ID_MAX];
  char authority_scope_ref[YAI_SOURCE_REF_MAX];
  char scope_state[YAI_SOURCE_STATUS_MAX];
  char suspension_state[YAI_SOURCE_STATUS_MAX];
  char revoke_state[YAI_SOURCE_STATUS_MAX];
  int64_t refreshed_at_epoch;
} yai_source_mesh_authority_scope_t;

typedef struct yai_source_mesh_transport_endpoint {
  char mesh_transport_endpoint_id[YAI_SOURCE_REF_MAX];
  char mesh_id[YAI_SOURCE_REF_MAX];
  char mesh_node_id[YAI_SOURCE_NODE_ID_MAX];
  char owner_workspace_id[YAI_SOURCE_WORKSPACE_ID_MAX];
  char endpoint_role[YAI_SOURCE_KIND_MAX];
  char endpoint_ref[YAI_SOURCE_REF_MAX];
  char endpoint_state[YAI_SOURCE_STATUS_MAX];
  char overlay_identity_ref[YAI_SOURCE_REF_MAX];
  int64_t observed_at_epoch;
} yai_source_mesh_transport_endpoint_t;

typedef struct yai_source_mesh_transport_path_state {
  char mesh_transport_path_state_id[YAI_SOURCE_REF_MAX];
  char mesh_id[YAI_SOURCE_REF_MAX];
  char owner_workspace_id[YAI_SOURCE_WORKSPACE_ID_MAX];
  char source_mesh_node_id[YAI_SOURCE_NODE_ID_MAX];
  char target_mesh_node_id[YAI_SOURCE_NODE_ID_MAX];
  char reachability_state[YAI_SOURCE_STATUS_MAX];
  char ingress_readiness[YAI_SOURCE_STATUS_MAX];
  char path_freshness[YAI_SOURCE_STATUS_MAX];
  char degradation_state[YAI_SOURCE_STATUS_MAX];
  char reconnect_state[YAI_SOURCE_STATUS_MAX];
  int64_t refreshed_at_epoch;
} yai_source_mesh_transport_path_state_t;

typedef struct yai_source_mesh_transport_channel_state {
  char mesh_transport_channel_state_id[YAI_SOURCE_REF_MAX];
  char mesh_id[YAI_SOURCE_REF_MAX];
  char source_mesh_node_id[YAI_SOURCE_NODE_ID_MAX];
  char target_mesh_node_id[YAI_SOURCE_NODE_ID_MAX];
  char secure_channel_state[YAI_SOURCE_STATUS_MAX];
  char channel_health[YAI_SOURCE_STATUS_MAX];
  char channel_kind[YAI_SOURCE_KIND_MAX];
  char reconnect_required[YAI_SOURCE_STATUS_MAX];
  int64_t updated_at_epoch;
} yai_source_mesh_transport_channel_state_t;

const char *yai_source_contract_operation_name(yai_source_contract_operation_t op);
int yai_source_record_class_is_known(const char *record_class);
