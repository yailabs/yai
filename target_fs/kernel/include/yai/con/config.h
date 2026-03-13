#pragma once

#include <stdint.h>

#define YAI_CONTAINER_CONFIG_REF_MAX 127u

typedef enum {
  YAI_CONTAINER_CREATION_POLICY_STANDARD = 0,
  YAI_CONTAINER_CREATION_POLICY_STRICT,
  YAI_CONTAINER_CREATION_POLICY_RECOVERY_FIRST,
} yai_container_creation_policy_t;

typedef enum {
  YAI_CONTAINER_ROOT_MODEL_PROJECTED = 0,
  YAI_CONTAINER_ROOT_MODEL_PROJECTED_STRICT,
} yai_container_root_model_t;

typedef enum {
  YAI_CONTAINER_MOUNT_POLICY_BASELINE = 0,
  YAI_CONTAINER_MOUNT_POLICY_RESTRICTED,
  YAI_CONTAINER_MOUNT_POLICY_PRIVILEGED,
} yai_container_mount_policy_profile_t;

typedef enum {
  YAI_CONTAINER_VISIBILITY_POLICY_DEFAULT = 0,
  YAI_CONTAINER_VISIBILITY_POLICY_RESTRICTED,
  YAI_CONTAINER_VISIBILITY_POLICY_STRICT,
} yai_container_visibility_policy_t;

typedef enum {
  YAI_CONTAINER_SESSION_POLICY_STANDARD = 0,
  YAI_CONTAINER_SESSION_POLICY_PRIVILEGED_ALLOWED,
  YAI_CONTAINER_SESSION_POLICY_RECOVERY_ONLY,
} yai_container_session_policy_t;

typedef enum {
  YAI_CONTAINER_SERVICES_PROFILE_MINIMAL = 0,
  YAI_CONTAINER_SERVICES_PROFILE_STANDARD,
  YAI_CONTAINER_SERVICES_PROFILE_EXTENDED,
} yai_container_services_profile_t;

typedef enum {
  YAI_CONTAINER_RECOVERY_POLICY_NONE = 0,
  YAI_CONTAINER_RECOVERY_POLICY_STANDARD,
  YAI_CONTAINER_RECOVERY_POLICY_STRICT,
} yai_container_recovery_policy_t;

typedef enum {
  YAI_CONTAINER_RESOURCE_PROFILE_DEFAULT = 0,
  YAI_CONTAINER_RESOURCE_PROFILE_LOW_LATENCY,
  YAI_CONTAINER_RESOURCE_PROFILE_ISOLATED,
} yai_container_resource_profile_t;

typedef enum {
  YAI_CONTAINER_DAEMON_ATTACHMENT_DENY = 0,
  YAI_CONTAINER_DAEMON_ATTACHMENT_CONTROLLED,
  YAI_CONTAINER_DAEMON_ATTACHMENT_PRIVILEGED,
} yai_container_daemon_attachment_policy_t;

typedef struct {
  char backing_store_ref[YAI_CONTAINER_CONFIG_REF_MAX + 1u];
  yai_container_root_model_t root_model;
  yai_container_mount_policy_profile_t mount_policy;
  yai_container_visibility_policy_t visibility_policy;
} yai_container_root_config_t;

typedef struct {
  yai_container_session_policy_t session_policy;
  uint64_t allowed_session_modes_mask;
  uint64_t interactive_flags;
} yai_container_session_config_t;

typedef struct {
  yai_container_services_profile_t services_profile;
  uint64_t required_service_mask;
  uint64_t readiness_minimum;
} yai_container_service_config_t;

typedef struct {
  yai_container_recovery_policy_t recovery_policy;
  uint64_t recoverable_mask;
} yai_container_recovery_config_t;

typedef struct {
  yai_container_resource_profile_t resource_profile;
  uint64_t limits_class;
  uint64_t isolation_profile;
} yai_container_resource_config_t;

typedef struct {
  yai_container_creation_policy_t creation_policy;
  yai_container_root_config_t root;
  yai_container_session_config_t session;
  yai_container_service_config_t services;
  yai_container_recovery_config_t recovery;
  yai_container_resource_config_t resources;
  yai_container_daemon_attachment_policy_t daemon_attachment_policy;
  uint64_t network_profile;
  uint64_t policy_profile;
  uint64_t grants_profile;
  uint64_t runtime_profile;
  uint64_t config_revision;
  int64_t applied_at;
} yai_container_config_t;

void yai_container_config_defaults(yai_container_config_t *config);
int yai_container_config_load(const char *container_id, yai_container_config_t *out_config);
int yai_container_config_validate(const yai_container_config_t *config);
int yai_container_config_apply(const char *container_id,
                               const yai_container_config_t *config,
                               int64_t applied_at);
