#pragma once

#include <stdint.h>

#include "state.h"

typedef enum {
  YAI_CONTAINER_RECOVERY_CLASS_NONE = 0,
  YAI_CONTAINER_RECOVERY_CLASS_SOFT_REBIND,
  YAI_CONTAINER_RECOVERY_CLASS_STATE_RELOAD,
  YAI_CONTAINER_RECOVERY_CLASS_ROOT_REPROJECT,
  YAI_CONTAINER_RECOVERY_CLASS_SERVICE_REHYDRATE,
  YAI_CONTAINER_RECOVERY_CLASS_FULL_DOMAIN,
} yai_container_recovery_class_t;

typedef enum {
  YAI_CONTAINER_RECOVERY_OUTCOME_NONE = 0,
  YAI_CONTAINER_RECOVERY_OUTCOME_RECOVERED_TO_OPEN,
  YAI_CONTAINER_RECOVERY_OUTCOME_RECOVERED_TO_ACTIVE,
  YAI_CONTAINER_RECOVERY_OUTCOME_RECOVERED_TO_DEGRADED,
  YAI_CONTAINER_RECOVERY_OUTCOME_UNRECOVERABLE,
  YAI_CONTAINER_RECOVERY_OUTCOME_SEALED_FOR_INSPECTION,
} yai_container_recovery_outcome_t;

typedef struct {
  yai_container_recovery_class_t recovery_class;
  yai_container_recovery_status_t recovery_status;
  yai_container_recovery_outcome_t last_outcome;
  uint64_t reason_flags;
  uint64_t marker_flags;
  uint64_t last_error_class;
  int64_t updated_at;
} yai_container_recovery_record_t;

int yai_container_enter_recovery(const char *container_id, uint64_t reason_flags);
int yai_container_seal(const char *container_id, int64_t sealed_at);
int yai_container_restore(const char *container_id);

int yai_container_recovery_check(const char *container_id,
                                 yai_container_recovery_status_t *out_status,
                                 yai_container_recovery_record_t *out_record);
int yai_container_recovery_prepare(const char *container_id,
                                   yai_container_recovery_class_t recovery_class,
                                   uint64_t reason_flags);
int yai_container_recovery_execute(const char *container_id,
                                   yai_container_recovery_class_t recovery_class);
int yai_container_recovery_complete(const char *container_id,
                                    yai_container_lifecycle_state_t target_state);
int yai_container_recovery_seal(const char *container_id, int64_t sealed_at);
