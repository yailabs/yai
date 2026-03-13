#include <string.h>

#include "internal/model.h"
#include "yai/container/config.h"

void yai_container_config_defaults(yai_container_config_t *config) {
  if (!config) {
    return;
  }

  memset(config, 0, sizeof(*config));
  config->creation_policy = YAI_CONTAINER_CREATION_POLICY_STANDARD;

  config->root.root_model = YAI_CONTAINER_ROOT_MODEL_PROJECTED;
  config->root.mount_policy = YAI_CONTAINER_MOUNT_POLICY_BASELINE;
  config->root.visibility_policy = YAI_CONTAINER_VISIBILITY_POLICY_DEFAULT;

  config->session.session_policy = YAI_CONTAINER_SESSION_POLICY_STANDARD;
  config->session.allowed_session_modes_mask = UINT64_MAX;

  config->services.services_profile = YAI_CONTAINER_SERVICES_PROFILE_STANDARD;
  config->services.readiness_minimum = 1;

  config->recovery.recovery_policy = YAI_CONTAINER_RECOVERY_POLICY_STANDARD;
  config->recovery.recoverable_mask = UINT64_MAX;

  config->resources.resource_profile = YAI_CONTAINER_RESOURCE_PROFILE_DEFAULT;
  config->resources.limits_class = 1;
  config->resources.isolation_profile = 1;

  config->daemon_attachment_policy = YAI_CONTAINER_DAEMON_ATTACHMENT_CONTROLLED;
  config->network_profile = 1;
  config->policy_profile = 1;
  config->grants_profile = 1;
  config->runtime_profile = 1;
  config->config_revision = 1;
  config->applied_at = 0;
}

int yai_container_config_validate(const yai_container_config_t *config) {
  if (!config) {
    return -1;
  }
  if (config->root.root_model > YAI_CONTAINER_ROOT_MODEL_PROJECTED_STRICT) {
    return -1;
  }
  if (config->root.mount_policy > YAI_CONTAINER_MOUNT_POLICY_PRIVILEGED) {
    return -1;
  }
  if (config->root.visibility_policy > YAI_CONTAINER_VISIBILITY_POLICY_STRICT) {
    return -1;
  }
  if (config->session.session_policy > YAI_CONTAINER_SESSION_POLICY_RECOVERY_ONLY) {
    return -1;
  }
  if (config->services.services_profile > YAI_CONTAINER_SERVICES_PROFILE_EXTENDED) {
    return -1;
  }
  if (config->recovery.recovery_policy > YAI_CONTAINER_RECOVERY_POLICY_STRICT) {
    return -1;
  }
  if (config->resources.resource_profile > YAI_CONTAINER_RESOURCE_PROFILE_ISOLATED) {
    return -1;
  }
  if (config->daemon_attachment_policy > YAI_CONTAINER_DAEMON_ATTACHMENT_PRIVILEGED) {
    return -1;
  }
  if (config->resources.limits_class == 0 || config->resources.isolation_profile == 0) {
    return -1;
  }
  if (config->policy_profile == 0 || config->grants_profile == 0 || config->runtime_profile == 0) {
    return -1;
  }
  return 0;
}

int yai_container_config_load(const char *container_id, yai_container_config_t *out_config) {
  yai_container_record_t record;

  if (!container_id || !out_config || container_id[0] == '\0') {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  *out_config = record.config;
  return 0;
}

int yai_container_config_apply(const char *container_id,
                               const yai_container_config_t *config,
                               int64_t applied_at) {
  yai_container_record_t record;
  yai_container_config_t next;

  if (!container_id || !config || container_id[0] == '\0') {
    return -1;
  }
  if (yai_container_config_validate(config) != 0) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }

  next = *config;
  if (next.config_revision == 0) {
    next.config_revision = (record.config.config_revision == 0) ? 1 : record.config.config_revision + 1;
  }
  next.applied_at = applied_at;
  record.config = next;
  record.state.updated_at = applied_at;

  return yai_container_model_upsert(&record);
}
