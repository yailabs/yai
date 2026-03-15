#include <time.h>

#include <string.h>

#include <yai/con/runtime.h>

int yai_container_root_projection_initialize(const char *container_id);
int yai_container_session_bind(const char *container_id, uint64_t session_id);

int yai_container_create(const yai_container_record_t *record) {
  yai_container_record_t normalized;
  int64_t now = (int64_t)time(NULL);

  if (!record || record->identity.container_id[0] == '\0') {
    return -1;
  }

  normalized = *record;

  if (normalized.lifecycle.current > YAI_CONTAINER_LIFECYCLE_ARCHIVED) {
    normalized.lifecycle.current = YAI_CONTAINER_LIFECYCLE_CREATED;
  }
  if (normalized.lifecycle.previous > YAI_CONTAINER_LIFECYCLE_ARCHIVED) {
    normalized.lifecycle.previous = YAI_CONTAINER_LIFECYCLE_CREATED;
  }

  if (normalized.state.lifecycle_state > YAI_CONTAINER_LIFECYCLE_ARCHIVED) {
    normalized.state.lifecycle_state = YAI_CONTAINER_LIFECYCLE_CREATED;
  }
  normalized.state.updated_at = now;

  if (normalized.root.container_root_handle == 0) {
    normalized.root.container_root_handle = normalized.identity.state_handle;
  }

  if (yai_container_registry_register(&normalized) != 0) {
    return -1;
  }

  if (yai_container_config_apply(normalized.identity.container_id, &normalized.config, now) != 0) {
    return -1;
  }

  return yai_container_state_init(normalized.identity.container_id, now);
}

int yai_container_open(const char *container_id) {
  return yai_container_registry_set_lifecycle(container_id,
                                              YAI_CONTAINER_LIFECYCLE_OPEN,
                                              (int64_t)time(NULL));
}

int yai_container_attach(const char *container_id, uint64_t session_id) {
  yai_container_record_t record;

  if (!container_id || container_id[0] == '\0' || session_id == 0) {
    return -1;
  }

  if (yai_container_registry_get_record(container_id, &record) != 0) {
    return -1;
  }

  if (record.lifecycle.current == YAI_CONTAINER_LIFECYCLE_INITIALIZED) {
    if (yai_container_open(container_id) != 0) {
      return -1;
    }
  } else if (record.lifecycle.current != YAI_CONTAINER_LIFECYCLE_OPEN &&
             record.lifecycle.current != YAI_CONTAINER_LIFECYCLE_ACTIVE) {
    return -1;
  }

  return yai_container_session_bind(container_id, session_id);
}

int yai_container_initialize(const char *container_id) {
  if (yai_container_registry_set_lifecycle(container_id,
                                           YAI_CONTAINER_LIFECYCLE_INITIALIZED,
                                           (int64_t)time(NULL)) != 0) {
    return -1;
  }
  if (yai_container_root_projection_initialize(container_id) != 0) {
    return -1;
  }
  return yai_container_state_init(container_id, (int64_t)time(NULL));
}

int yai_container_recover(const char *container_id, uint64_t reason_flags) {
  if (yai_container_recovery_prepare(container_id,
                                     YAI_CONTAINER_RECOVERY_CLASS_STATE_RELOAD,
                                     reason_flags) != 0) {
    return -1;
  }
  if (yai_container_recovery_execute(container_id, YAI_CONTAINER_RECOVERY_CLASS_STATE_RELOAD) != 0) {
    return -1;
  }
  return yai_container_recovery_complete(container_id, YAI_CONTAINER_LIFECYCLE_ACTIVE);
}

int yai_container_seal_runtime(const char *container_id, int64_t sealed_at) {
  return yai_container_seal(container_id, sealed_at);
}

int yai_container_destroy(const char *container_id, int64_t destroyed_at) {
  return yai_container_registry_set_lifecycle(container_id,
                                              YAI_CONTAINER_LIFECYCLE_DESTROYED,
                                              destroyed_at);
}
