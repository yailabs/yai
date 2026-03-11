#include <string.h>

#include <yai/daemon/source_plane_model.h>

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
         strcmp(record_class, YAI_SOURCE_RECORD_CLASS_MESH_AUTHORITY_SCOPE) == 0;
}
