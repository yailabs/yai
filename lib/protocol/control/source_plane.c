#include <string.h>

#include <yai/protocol/control/source_plane.h>

static const yai_source_contract_shape_t SHAPES[] = {
    {YAI_SOURCE_CONTRACT_ENROLL, YAI_SOURCE_CONTRACT_TYPE_ENROLL_CALL, YAI_SOURCE_CONTRACT_TYPE_ENROLL_REPLY},
    {YAI_SOURCE_CONTRACT_ATTACH, YAI_SOURCE_CONTRACT_TYPE_ATTACH_CALL, YAI_SOURCE_CONTRACT_TYPE_ATTACH_REPLY},
    {YAI_SOURCE_CONTRACT_EMIT, YAI_SOURCE_CONTRACT_TYPE_EMIT_CALL, YAI_SOURCE_CONTRACT_TYPE_EMIT_REPLY},
    {YAI_SOURCE_CONTRACT_STATUS, YAI_SOURCE_CONTRACT_TYPE_STATUS_CALL, YAI_SOURCE_CONTRACT_TYPE_STATUS_REPLY},
};

const char *yai_source_contract_operation_name(yai_source_contract_operation_t op)
{
  switch (op)
  {
  case YAI_SOURCE_CONTRACT_ENROLL:
    return "enroll";
  case YAI_SOURCE_CONTRACT_ATTACH:
    return "attach";
  case YAI_SOURCE_CONTRACT_EMIT:
    return "emit";
  case YAI_SOURCE_CONTRACT_STATUS:
    return "status";
  default:
    return "invalid";
  }
}

int yai_source_record_class_is_known(const char *record_class)
{
  if (!record_class || !record_class[0])
  {
    return 0;
  }
  return strcmp(record_class, YAI_SOURCE_RECORD_CLASS_NODE) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_DAEMON_INSTANCE) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_BINDING) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_ASSET) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_ACQUISITION_EVENT) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_EVIDENCE_CANDIDATE) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_ACTION_POINT) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_OWNER_LINK) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_ENROLLMENT_GRANT) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_POLICY_SNAPSHOT) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_CAPABILITY_ENVELOPE) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_WORKSPACE_PEER_MEMBERSHIP) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_INGEST_OUTCOME) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_NODE) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_DISCOVERY_ADVERTISEMENT) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_BOOTSTRAP_DESCRIPTOR) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_COORDINATION_MEMBERSHIP) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_PEER_AWARENESS) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_PEER_LEGITIMACY) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_AUTHORITY_SCOPE) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_TRANSPORT_ENDPOINT) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_TRANSPORT_PATH_STATE) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_TRANSPORT_CHANNEL_STATE) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_OWNER_REMOTE_INGRESS) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_OWNER_REMOTE_INGRESS_SESSION) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_OWNER_REMOTE_INGRESS_DECISION) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_OVERLAY_PRESENCE) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_OVERLAY_TARGET_ASSOCIATION) == 0 ||
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_OVERLAY_PATH_MUTATION) == 0;
}

const yai_source_contract_shape_t *yai_source_contract_shape(yai_source_contract_operation_t op)
{
  size_t i = 0;
  for (i = 0; i < (sizeof(SHAPES) / sizeof(SHAPES[0])); ++i)
  {
    if (SHAPES[i].op == op)
    {
      return &SHAPES[i];
    }
  }
  return NULL;
}
