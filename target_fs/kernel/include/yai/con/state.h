#pragma once

#include <stdint.h>

#include "lifecycle.h"

typedef enum {
  YAI_CONTAINER_RUNTIME_STATE_NONE = 0,
  YAI_CONTAINER_RUNTIME_STATE_BOOTSTRAPPING,
  YAI_CONTAINER_RUNTIME_STATE_READY,
  YAI_CONTAINER_RUNTIME_STATE_DEGRADED,
  YAI_CONTAINER_RUNTIME_STATE_FAILED,
} yai_container_runtime_state_t;

typedef enum {
  YAI_CONTAINER_ROOT_STATUS_NONE = 0,
  YAI_CONTAINER_ROOT_STATUS_PROJECTED,
  YAI_CONTAINER_ROOT_STATUS_PARTIAL,
  YAI_CONTAINER_ROOT_STATUS_FAILED,
} yai_container_root_status_t;

typedef enum {
  YAI_CONTAINER_MOUNT_STATUS_NONE = 0,
  YAI_CONTAINER_MOUNT_STATUS_APPLIED,
  YAI_CONTAINER_MOUNT_STATUS_PARTIAL,
  YAI_CONTAINER_MOUNT_STATUS_FAILED,
} yai_container_mount_status_t;

typedef enum {
  YAI_CONTAINER_SESSION_STATUS_NONE = 0,
  YAI_CONTAINER_SESSION_STATUS_ACTIVE,
  YAI_CONTAINER_SESSION_STATUS_PRIVILEGED_PRESENT,
  YAI_CONTAINER_SESSION_STATUS_BINDING_DEGRADED,
} yai_container_session_status_t;

typedef enum {
  YAI_CONTAINER_SERVICE_STATUS_NONE = 0,
  YAI_CONTAINER_SERVICE_STATUS_NOT_INITIALIZED,
  YAI_CONTAINER_SERVICE_STATUS_PARTIALLY_READY,
  YAI_CONTAINER_SERVICE_STATUS_READY,
  YAI_CONTAINER_SERVICE_STATUS_DEGRADED,
} yai_container_service_status_t;

typedef enum {
  YAI_CONTAINER_RECOVERY_STATUS_NONE = 0,
  YAI_CONTAINER_RECOVERY_STATUS_RESTORABLE,
  YAI_CONTAINER_RECOVERY_STATUS_RECOVERY_REQUIRED,
  YAI_CONTAINER_RECOVERY_STATUS_RECOVERING,
  YAI_CONTAINER_RECOVERY_STATUS_UNRECOVERABLE,
} yai_container_recovery_status_t;

typedef enum {
  YAI_CONTAINER_HEALTH_NOMINAL = 0,
  YAI_CONTAINER_HEALTH_WARNING,
  YAI_CONTAINER_HEALTH_DEGRADED,
  YAI_CONTAINER_HEALTH_FAILED,
} yai_container_health_state_t;

typedef struct {
  uint64_t health_flags;
  uint64_t resource_class;
  uint64_t attachments;
  uint64_t services_online;
  uint64_t daemon_bindings;
  uint64_t network_bindings;
  uint64_t recovery_reason_flags;
  uint64_t degraded_flags;
  uint64_t last_error_class;
  int64_t created_at;
  int64_t updated_at;
  yai_container_lifecycle_state_t lifecycle_state;
  yai_container_runtime_state_t runtime_state;
  yai_container_root_status_t root_status;
  yai_container_mount_status_t mount_status;
  yai_container_session_status_t session_status;
  yai_container_service_status_t service_status;
  yai_container_recovery_status_t recovery_status;
  yai_container_health_state_t health_state;
} yai_container_state_t;

typedef struct {
  yai_container_state_t state;
  int64_t captured_at;
  char snapshot_id[64];
} yai_container_state_snapshot_t;

void yai_container_state_defaults(yai_container_state_t *state);
int yai_container_state_init(const char *container_id, int64_t created_at);
int yai_container_state_update(const char *container_id, const yai_container_state_t *state);
int yai_container_state_read(const char *container_id, yai_container_state_t *out_state);
int yai_container_state_snapshot(const char *container_id,
                                 const char *snapshot_id,
                                 yai_container_state_snapshot_t *out_snapshot);
