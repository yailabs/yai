#define _POSIX_C_SOURCE 200809L

#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>

#include "internal/model.h"
#include <yai/con/state.h>

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

static int state_dir_path(const char *container_id, char *out, size_t out_cap) {
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
  return snprintf(out, out_cap, "%s/state", record_path) < (int)out_cap ? 0 : -1;
}

static int snapshot_path(const char *container_id,
                         const char *snapshot_id,
                         char *out,
                         size_t out_cap) {
  char dir[4096];

  if (!snapshot_id || snapshot_id[0] == '\0' || !out || out_cap == 0) {
    return -1;
  }
  if (state_dir_path(container_id, dir, sizeof(dir)) != 0) {
    return -1;
  }
  if (ensure_dir(dir) != 0) {
    return -1;
  }
  return snprintf(out, out_cap, "%s/%s.v1.bin", dir, snapshot_id) < (int)out_cap ? 0 : -1;
}

static yai_container_runtime_state_t runtime_state_from_lifecycle(
    yai_container_lifecycle_state_t lifecycle) {
  switch (lifecycle) {
    case YAI_CONTAINER_LIFECYCLE_CREATED:
    case YAI_CONTAINER_LIFECYCLE_INITIALIZED:
      return YAI_CONTAINER_RUNTIME_STATE_BOOTSTRAPPING;
    case YAI_CONTAINER_LIFECYCLE_OPEN:
    case YAI_CONTAINER_LIFECYCLE_ACTIVE:
      return YAI_CONTAINER_RUNTIME_STATE_READY;
    case YAI_CONTAINER_LIFECYCLE_DEGRADED:
    case YAI_CONTAINER_LIFECYCLE_RECOVERY:
      return YAI_CONTAINER_RUNTIME_STATE_DEGRADED;
    case YAI_CONTAINER_LIFECYCLE_SEALED:
    case YAI_CONTAINER_LIFECYCLE_DESTROYED:
    case YAI_CONTAINER_LIFECYCLE_ARCHIVED:
      return YAI_CONTAINER_RUNTIME_STATE_FAILED;
    default:
      return YAI_CONTAINER_RUNTIME_STATE_NONE;
  }
}

static int state_validate(const yai_container_state_t *state) {
  if (!state) {
    return -1;
  }
  if (state->lifecycle_state > YAI_CONTAINER_LIFECYCLE_ARCHIVED) {
    return -1;
  }
  if (state->runtime_state > YAI_CONTAINER_RUNTIME_STATE_FAILED) {
    return -1;
  }
  if (state->root_status > YAI_CONTAINER_ROOT_STATUS_FAILED) {
    return -1;
  }
  if (state->mount_status > YAI_CONTAINER_MOUNT_STATUS_FAILED) {
    return -1;
  }
  if (state->session_status > YAI_CONTAINER_SESSION_STATUS_BINDING_DEGRADED) {
    return -1;
  }
  if (state->service_status > YAI_CONTAINER_SERVICE_STATUS_DEGRADED) {
    return -1;
  }
  if (state->recovery_status > YAI_CONTAINER_RECOVERY_STATUS_UNRECOVERABLE) {
    return -1;
  }
  if (state->health_state > YAI_CONTAINER_HEALTH_FAILED) {
    return -1;
  }
  return 0;
}

void yai_container_state_defaults(yai_container_state_t *state) {
  if (!state) {
    return;
  }

  memset(state, 0, sizeof(*state));
  state->lifecycle_state = YAI_CONTAINER_LIFECYCLE_CREATED;
  state->runtime_state = YAI_CONTAINER_RUNTIME_STATE_BOOTSTRAPPING;
  state->root_status = YAI_CONTAINER_ROOT_STATUS_NONE;
  state->mount_status = YAI_CONTAINER_MOUNT_STATUS_NONE;
  state->session_status = YAI_CONTAINER_SESSION_STATUS_NONE;
  state->service_status = YAI_CONTAINER_SERVICE_STATUS_NOT_INITIALIZED;
  state->recovery_status = YAI_CONTAINER_RECOVERY_STATUS_NONE;
  state->health_state = YAI_CONTAINER_HEALTH_WARNING;
}

int yai_container_state_init(const char *container_id, int64_t created_at) {
  yai_container_record_t record;
  yai_container_state_t state;

  if (!container_id || container_id[0] == '\0') {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }

  yai_container_state_defaults(&state);
  state.created_at = created_at;
  state.updated_at = created_at;
  state.lifecycle_state = record.lifecycle.current;
  state.runtime_state = runtime_state_from_lifecycle(record.lifecycle.current);
  state.root_status = record.root.projection_ready ? YAI_CONTAINER_ROOT_STATUS_PROJECTED
                                                   : YAI_CONTAINER_ROOT_STATUS_NONE;
  state.mount_status = YAI_CONTAINER_MOUNT_STATUS_NONE;
  state.session_status = record.session_domain.bound
                             ? (record.session_domain.privileged_access
                                    ? YAI_CONTAINER_SESSION_STATUS_PRIVILEGED_PRESENT
                                    : YAI_CONTAINER_SESSION_STATUS_ACTIVE)
                             : YAI_CONTAINER_SESSION_STATUS_NONE;
  state.service_status = YAI_CONTAINER_SERVICE_STATUS_NOT_INITIALIZED;
  state.health_state = (state.root_status == YAI_CONTAINER_ROOT_STATUS_PROJECTED)
                           ? YAI_CONTAINER_HEALTH_NOMINAL
                           : YAI_CONTAINER_HEALTH_WARNING;

  record.state = state;
  return yai_container_model_upsert(&record);
}

int yai_container_state_update(const char *container_id, const yai_container_state_t *state) {
  yai_container_record_t record;

  if (!container_id || !state || container_id[0] == '\0') {
    return -1;
  }
  if (state_validate(state) != 0) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }

  record.state = *state;
  return yai_container_model_upsert(&record);
}

int yai_container_state_read(const char *container_id, yai_container_state_t *out_state) {
  yai_container_record_t record;

  if (!container_id || !out_state || container_id[0] == '\0') {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  *out_state = record.state;
  return 0;
}

int yai_container_state_snapshot(const char *container_id,
                                 const char *snapshot_id,
                                 yai_container_state_snapshot_t *out_snapshot) {
  yai_container_state_snapshot_t snapshot;
  char path[4096];
  FILE *fp = NULL;

  if (!container_id || !snapshot_id || !out_snapshot || container_id[0] == '\0') {
    return -1;
  }
  if (yai_container_state_read(container_id, &snapshot.state) != 0) {
    return -1;
  }
  snapshot.captured_at = snapshot.state.updated_at;
  if (snapshot.captured_at == 0) {
    snapshot.captured_at = snapshot.state.created_at;
  }
  if (snprintf(snapshot.snapshot_id, sizeof(snapshot.snapshot_id), "%s", snapshot_id) >=
      (int)sizeof(snapshot.snapshot_id)) {
    return -1;
  }

  if (snapshot_path(container_id, snapshot_id, path, sizeof(path)) != 0) {
    return -1;
  }

  fp = fopen(path, "wb");
  if (!fp) {
    return -1;
  }
  if (fwrite(&snapshot, sizeof(snapshot), 1, fp) != 1) {
    fclose(fp);
    return -1;
  }
  if (fclose(fp) != 0) {
    return -1;
  }

  *out_snapshot = snapshot;
  return 0;
}
