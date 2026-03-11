#pragma once

#include <stddef.h>
#include <stdint.h>

#define YAI_SOURCE_NODE_ID_MAX 96
#define YAI_SOURCE_DAEMON_INSTANCE_ID_MAX 96
#define YAI_SOURCE_BINDING_ID_MAX 96
#define YAI_SOURCE_ASSET_ID_MAX 128
#define YAI_SOURCE_ACQUISITION_EVENT_ID_MAX 128
#define YAI_SOURCE_EVIDENCE_CANDIDATE_ID_MAX 128
#define YAI_SOURCE_OWNER_LINK_ID_MAX 96
#define YAI_SOURCE_ENROLLMENT_GRANT_ID_MAX 128
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
#define YAI_SOURCE_RECORD_CLASS_OWNER_LINK "source_owner_link"
#define YAI_SOURCE_RECORD_CLASS_ENROLLMENT_GRANT "source_enrollment_grant"

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
  int64_t issued_at_epoch;
} yai_source_enrollment_grant_t;

const char *yai_source_contract_operation_name(yai_source_contract_operation_t op);
int yai_source_record_class_is_known(const char *record_class);
