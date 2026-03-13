#define _POSIX_C_SOURCE 200809L

#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>

#include "../runtime/internal/model.h"
#include "yai/container/recovery.h"
#include "yai/container/registry.h"
#include "yai/container/root.h"
#include "yai/container/state.h"

static int ensure_dir(const char *path) {
  struct stat st;

  if (!path || path[0] == '\0') {
    return -1;
  }
  if (stat(path, &st) == 0) {
    return S_ISDIR(st.st_mode) ? 0 : -1;
  }
  if (mkdir(path, 0755) == 0) {
    return 0;
  }
  return errno == EEXIST ? 0 : -1;
}

static int recovery_path(const char *container_id, char *out, size_t out_cap) {
  char record_path[4096];
  char *slash;

  if (!container_id || !out || out_cap == 0) {
    return -1;
  }
  if (yai_container_model_record_path(container_id, record_path, sizeof(record_path)) != 0) {
    return -1;
  }
  slash = strrchr(record_path, '/');
  if (!slash) {
    return -1;
  }
  *slash = '\0';
  if (ensure_dir(record_path) != 0) {
    return -1;
  }
  return snprintf(out, out_cap, "%s/recovery.v1.bin", record_path) < (int)out_cap ? 0 : -1;
}

static int recovery_record_load(const char *container_id, yai_container_recovery_record_t *out) {
  char path[4096];
  FILE *fp = NULL;

  if (!container_id || !out || container_id[0] == '\0') {
    return -1;
  }
  if (recovery_path(container_id, path, sizeof(path)) != 0) {
    return -1;
  }

  memset(out, 0, sizeof(*out));
  out->recovery_class = YAI_CONTAINER_RECOVERY_CLASS_NONE;
  out->recovery_status = YAI_CONTAINER_RECOVERY_STATUS_NONE;
  out->last_outcome = YAI_CONTAINER_RECOVERY_OUTCOME_NONE;

  fp = fopen(path, "rb");
  if (!fp) {
    return 0;
  }
  if (fread(out, sizeof(*out), 1, fp) != 1) {
    fclose(fp);
    return -1;
  }
  return fclose(fp) == 0 ? 0 : -1;
}

static int recovery_record_save(const char *container_id, const yai_container_recovery_record_t *rec) {
  char path[4096];
  FILE *fp = NULL;

  if (!container_id || !rec || container_id[0] == '\0') {
    return -1;
  }
  if (recovery_path(container_id, path, sizeof(path)) != 0) {
    return -1;
  }
  fp = fopen(path, "wb");
  if (!fp) {
    return -1;
  }
  if (fwrite(rec, sizeof(*rec), 1, fp) != 1) {
    fclose(fp);
    return -1;
  }
  return fclose(fp) == 0 ? 0 : -1;
}

static yai_container_recovery_outcome_t outcome_from_target(yai_container_lifecycle_state_t target_state) {
  switch (target_state) {
    case YAI_CONTAINER_LIFECYCLE_OPEN:
      return YAI_CONTAINER_RECOVERY_OUTCOME_RECOVERED_TO_OPEN;
    case YAI_CONTAINER_LIFECYCLE_ACTIVE:
      return YAI_CONTAINER_RECOVERY_OUTCOME_RECOVERED_TO_ACTIVE;
    case YAI_CONTAINER_LIFECYCLE_DEGRADED:
      return YAI_CONTAINER_RECOVERY_OUTCOME_RECOVERED_TO_DEGRADED;
    default:
      return YAI_CONTAINER_RECOVERY_OUTCOME_NONE;
  }
}

static uint64_t marker_flags_for_record(const yai_container_record_t *record) {
  uint64_t flags = 0;

  if (!record) {
    return 0;
  }
  if (yai_container_config_validate(&record->config) == 0) {
    flags |= (1ull << 0); /* config marker */
  }
  flags |= (1ull << 1); /* state marker exists if record exists */
  if (record->root.projection_ready) {
    flags |= (1ull << 2); /* root marker */
  }
  if (record->state.mount_status == YAI_CONTAINER_MOUNT_STATUS_APPLIED) {
    flags |= (1ull << 3); /* mounts marker */
  }
  if (record->state.service_status == YAI_CONTAINER_SERVICE_STATUS_READY ||
      record->state.service_status == YAI_CONTAINER_SERVICE_STATUS_PARTIALLY_READY) {
    flags |= (1ull << 4); /* service marker */
  }
  if (record->config.policy_profile != 0 && record->config.grants_profile != 0) {
    flags |= (1ull << 5); /* policy/grants view marker */
  }
  return flags;
}

int yai_container_recovery_check(const char *container_id,
                                 yai_container_recovery_status_t *out_status,
                                 yai_container_recovery_record_t *out_record) {
  yai_container_record_t record;
  yai_container_recovery_record_t rec;

  if (!container_id || container_id[0] == '\0') {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  if (recovery_record_load(container_id, &rec) != 0) {
    return -1;
  }

  if (rec.recovery_status == YAI_CONTAINER_RECOVERY_STATUS_NONE) {
    if (record.lifecycle.current == YAI_CONTAINER_LIFECYCLE_RECOVERY) {
      rec.recovery_status = YAI_CONTAINER_RECOVERY_STATUS_RECOVERING;
    } else if (record.lifecycle.current == YAI_CONTAINER_LIFECYCLE_DEGRADED) {
      rec.recovery_status = YAI_CONTAINER_RECOVERY_STATUS_RECOVERY_REQUIRED;
    } else {
      rec.recovery_status = YAI_CONTAINER_RECOVERY_STATUS_RESTORABLE;
    }
  }
  rec.marker_flags = marker_flags_for_record(&record);

  if (out_status) {
    *out_status = rec.recovery_status;
  }
  if (out_record) {
    *out_record = rec;
  }
  return 0;
}

int yai_container_recovery_prepare(const char *container_id,
                                   yai_container_recovery_class_t recovery_class,
                                   uint64_t reason_flags) {
  yai_container_record_t record;
  yai_container_recovery_record_t rec;
  int64_t now = (int64_t)time(NULL);

  if (!container_id || container_id[0] == '\0') {
    return -1;
  }
  if (recovery_class > YAI_CONTAINER_RECOVERY_CLASS_FULL_DOMAIN) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  if (recovery_record_load(container_id, &rec) != 0) {
    return -1;
  }

  rec.recovery_class = recovery_class;
  rec.recovery_status = YAI_CONTAINER_RECOVERY_STATUS_RECOVERY_REQUIRED;
  rec.last_outcome = YAI_CONTAINER_RECOVERY_OUTCOME_NONE;
  rec.reason_flags = reason_flags;
  rec.marker_flags = marker_flags_for_record(&record);
  rec.last_error_class = record.state.last_error_class;
  rec.updated_at = now;

  record.state.recovery_reason_flags = reason_flags;
  record.state.recovery_status = YAI_CONTAINER_RECOVERY_STATUS_RECOVERY_REQUIRED;
  record.state.health_state = YAI_CONTAINER_HEALTH_DEGRADED;
  record.state.runtime_state = YAI_CONTAINER_RUNTIME_STATE_DEGRADED;
  record.state.updated_at = now;

  if (record.lifecycle.current == YAI_CONTAINER_LIFECYCLE_ACTIVE ||
      record.lifecycle.current == YAI_CONTAINER_LIFECYCLE_OPEN ||
      record.lifecycle.current == YAI_CONTAINER_LIFECYCLE_INITIALIZED) {
    if (yai_container_registry_set_lifecycle(container_id,
                                             YAI_CONTAINER_LIFECYCLE_DEGRADED,
                                             now) != 0) {
      return -1;
    }
    if (yai_container_model_get(container_id, &record) != 0) {
      return -1;
    }
    record.state.recovery_reason_flags = reason_flags;
    record.state.recovery_status = YAI_CONTAINER_RECOVERY_STATUS_RECOVERY_REQUIRED;
    record.state.health_state = YAI_CONTAINER_HEALTH_DEGRADED;
    record.state.runtime_state = YAI_CONTAINER_RUNTIME_STATE_DEGRADED;
    record.state.updated_at = now;
  }

  if (yai_container_model_upsert(&record) != 0) {
    return -1;
  }
  return recovery_record_save(container_id, &rec);
}

int yai_container_recovery_execute(const char *container_id,
                                   yai_container_recovery_class_t recovery_class) {
  yai_container_record_t record;
  yai_container_recovery_record_t rec;
  int64_t now = (int64_t)time(NULL);

  if (!container_id || container_id[0] == '\0') {
    return -1;
  }
  if (recovery_class > YAI_CONTAINER_RECOVERY_CLASS_FULL_DOMAIN) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  if (recovery_record_load(container_id, &rec) != 0) {
    return -1;
  }

  if (record.lifecycle.current != YAI_CONTAINER_LIFECYCLE_RECOVERY) {
    if (record.lifecycle.current == YAI_CONTAINER_LIFECYCLE_DEGRADED) {
      if (yai_container_registry_set_lifecycle(container_id,
                                               YAI_CONTAINER_LIFECYCLE_RECOVERY,
                                               now) != 0) {
        return -1;
      }
      if (yai_container_model_get(container_id, &record) != 0) {
        return -1;
      }
    } else {
      return -1;
    }
  }

  if (recovery_class == YAI_CONTAINER_RECOVERY_CLASS_ROOT_REPROJECT ||
      recovery_class == YAI_CONTAINER_RECOVERY_CLASS_FULL_DOMAIN) {
    if (yai_container_root_projection_initialize(container_id) != 0) {
      return -1;
    }
    if (yai_container_model_get(container_id, &record) != 0) {
      return -1;
    }
  }

  rec.recovery_class = recovery_class;
  rec.recovery_status = YAI_CONTAINER_RECOVERY_STATUS_RECOVERING;
  rec.marker_flags = marker_flags_for_record(&record);
  rec.updated_at = now;

  record.state.recovery_status = YAI_CONTAINER_RECOVERY_STATUS_RECOVERING;
  record.state.runtime_state = YAI_CONTAINER_RUNTIME_STATE_DEGRADED;
  record.state.health_state = YAI_CONTAINER_HEALTH_WARNING;
  record.state.updated_at = now;

  if (yai_container_model_upsert(&record) != 0) {
    return -1;
  }
  return recovery_record_save(container_id, &rec);
}

int yai_container_recovery_complete(const char *container_id,
                                    yai_container_lifecycle_state_t target_state) {
  yai_container_record_t record;
  yai_container_recovery_record_t rec;
  int64_t now = (int64_t)time(NULL);

  if (!container_id || container_id[0] == '\0') {
    return -1;
  }
  if (target_state != YAI_CONTAINER_LIFECYCLE_OPEN &&
      target_state != YAI_CONTAINER_LIFECYCLE_ACTIVE &&
      target_state != YAI_CONTAINER_LIFECYCLE_DEGRADED) {
    return -1;
  }

  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  if (recovery_record_load(container_id, &rec) != 0) {
    return -1;
  }
  if (record.lifecycle.current != YAI_CONTAINER_LIFECYCLE_RECOVERY &&
      record.lifecycle.current != YAI_CONTAINER_LIFECYCLE_DEGRADED) {
    return -1;
  }

  if (record.lifecycle.current != target_state) {
    if (yai_container_registry_set_lifecycle(container_id, target_state, now) != 0) {
      return -1;
    }
    if (yai_container_model_get(container_id, &record) != 0) {
      return -1;
    }
  }

  rec.last_outcome = outcome_from_target(target_state);
  rec.recovery_status = (target_state == YAI_CONTAINER_LIFECYCLE_DEGRADED)
                            ? YAI_CONTAINER_RECOVERY_STATUS_RECOVERY_REQUIRED
                            : YAI_CONTAINER_RECOVERY_STATUS_RESTORABLE;
  rec.marker_flags = marker_flags_for_record(&record);
  rec.updated_at = now;

  record.state.recovery_status = rec.recovery_status;
  record.state.recovery_reason_flags = 0;
  record.state.runtime_state = (target_state == YAI_CONTAINER_LIFECYCLE_DEGRADED)
                                   ? YAI_CONTAINER_RUNTIME_STATE_DEGRADED
                                   : YAI_CONTAINER_RUNTIME_STATE_READY;
  record.state.health_state = (target_state == YAI_CONTAINER_LIFECYCLE_DEGRADED)
                                  ? YAI_CONTAINER_HEALTH_DEGRADED
                                  : YAI_CONTAINER_HEALTH_NOMINAL;
  record.state.updated_at = now;

  if (yai_container_model_upsert(&record) != 0) {
    return -1;
  }
  return recovery_record_save(container_id, &rec);
}

int yai_container_recovery_seal(const char *container_id, int64_t sealed_at) {
  yai_container_recovery_record_t rec;

  if (!container_id || container_id[0] == '\0') {
    return -1;
  }
  if (recovery_record_load(container_id, &rec) != 0) {
    return -1;
  }
  rec.last_outcome = YAI_CONTAINER_RECOVERY_OUTCOME_SEALED_FOR_INSPECTION;
  rec.recovery_status = YAI_CONTAINER_RECOVERY_STATUS_UNRECOVERABLE;
  rec.updated_at = sealed_at;
  if (recovery_record_save(container_id, &rec) != 0) {
    return -1;
  }
  return yai_container_registry_set_lifecycle(container_id,
                                              YAI_CONTAINER_LIFECYCLE_SEALED,
                                              sealed_at);
}

int yai_container_enter_recovery(const char *container_id, uint64_t reason_flags) {
  if (yai_container_recovery_prepare(container_id,
                                     YAI_CONTAINER_RECOVERY_CLASS_STATE_RELOAD,
                                     reason_flags) != 0) {
    return -1;
  }
  return yai_container_recovery_execute(container_id, YAI_CONTAINER_RECOVERY_CLASS_STATE_RELOAD);
}

int yai_container_seal(const char *container_id, int64_t sealed_at) {
  return yai_container_recovery_seal(container_id, sealed_at);
}

int yai_container_restore(const char *container_id) {
  return yai_container_recovery_complete(container_id, YAI_CONTAINER_LIFECYCLE_ACTIVE);
}
