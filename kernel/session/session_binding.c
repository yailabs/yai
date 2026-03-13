#include <string.h>
#include <time.h>
#include <stdio.h>
#include <stdint.h>

#include "../runtime/internal/model.h"
#include "yai/container/paths.h"
#include "yai/container/recovery.h"
#include "yai/container/registry.h"
#include "yai/container/runtime_view.h"
#include "yai/container/session.h"

static int mode_privileged(yai_container_session_mode_t mode) {
  return (mode == YAI_CONTAINER_SESSION_MODE_PRIVILEGED ||
          mode == YAI_CONTAINER_SESSION_MODE_RECOVERY ||
          mode == YAI_CONTAINER_SESSION_MODE_DIAGNOSTIC) ? 1 : 0;
}

static yai_container_escape_policy_class_t escape_policy_from_mode(yai_container_session_mode_t mode) {
  switch (mode) {
    case YAI_CONTAINER_SESSION_MODE_PRIVILEGED:
      return YAI_CONTAINER_ESCAPE_CONTROLLED_ADMIN;
    case YAI_CONTAINER_SESSION_MODE_RECOVERY:
      return YAI_CONTAINER_ESCAPE_RECOVERY;
    case YAI_CONTAINER_SESSION_MODE_DIAGNOSTIC:
      return YAI_CONTAINER_ESCAPE_DEBUG;
    case YAI_CONTAINER_SESSION_MODE_NORMAL:
    case YAI_CONTAINER_SESSION_MODE_GLOBAL:
    default:
      return YAI_CONTAINER_ESCAPE_NONE;
  }
}

static int session_bound_to(const yai_container_record_t *record, uint64_t session_id) {
  if (!record || session_id == 0) {
    return 0;
  }
  return (record->session_domain.bound &&
          record->session_domain.active_session_id == session_id) ? 1 : 0;
}

static int write_bound_state(yai_container_record_t *record,
                             uint64_t session_id,
                             yai_container_session_mode_t mode,
                             uint64_t capability_mask,
                             uint64_t interactive_flags,
                             int64_t now) {
  if (!record || session_id == 0) {
    return -1;
  }

  record->session_domain.bound = 1;
  record->session_domain.active_session_id = session_id;
  record->session_domain.bound_session_count += 1;
  record->session_domain.last_bound_session_id = session_id;
  record->session_domain.last_bound_at = (uint64_t)now;
  record->session_domain.session_mode = mode;
  record->session_domain.capability_mask = capability_mask;
  record->session_domain.interactive_flags = interactive_flags;
  record->session_domain.escape_policy_class = escape_policy_from_mode(mode);
  record->session_domain.privileged_access = (uint8_t)mode_privileged(mode);
  record->session_domain.root_handle = record->root.root_projection_handle;
  record->session_domain.runtime_view_handle = record->identity.state_handle;
  record->state.session_status = mode_privileged(mode)
                                     ? YAI_CONTAINER_SESSION_STATUS_PRIVILEGED_PRESENT
                                     : YAI_CONTAINER_SESSION_STATUS_ACTIVE;
  if (record->state.runtime_state == YAI_CONTAINER_RUNTIME_STATE_BOOTSTRAPPING) {
    record->state.runtime_state = YAI_CONTAINER_RUNTIME_STATE_READY;
  }
  if (record->state.health_state == YAI_CONTAINER_HEALTH_WARNING) {
    record->state.health_state = YAI_CONTAINER_HEALTH_NOMINAL;
  }
  record->state.updated_at = now;
  return 0;
}

int yai_container_bind_session(const char *container_id,
                               uint64_t session_id,
                               yai_container_session_mode_t mode,
                               uint64_t capability_mask,
                               uint64_t interactive_flags) {
  yai_container_record_t record;
  int64_t now = (int64_t)time(NULL);

  if (!container_id || container_id[0] == '\0' || session_id == 0) {
    return -1;
  }
  if (mode > YAI_CONTAINER_SESSION_MODE_DIAGNOSTIC) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  if (!record.root.projection_ready) {
    return -1;
  }

  if (record.lifecycle.current == YAI_CONTAINER_LIFECYCLE_OPEN ||
      record.lifecycle.current == YAI_CONTAINER_LIFECYCLE_INITIALIZED) {
    if (yai_container_registry_set_lifecycle(container_id,
                                             YAI_CONTAINER_LIFECYCLE_ACTIVE,
                                             now) != 0) {
      return -1;
    }
    if (yai_container_model_get(container_id, &record) != 0) {
      return -1;
    }
  } else if (record.lifecycle.current != YAI_CONTAINER_LIFECYCLE_ACTIVE &&
             record.lifecycle.current != YAI_CONTAINER_LIFECYCLE_RECOVERY) {
    return -1;
  }

  if (write_bound_state(&record, session_id, mode, capability_mask, interactive_flags, now) != 0) {
    return -1;
  }
  return yai_container_model_upsert(&record);
}

int yai_container_unbind_session(const char *container_id, uint64_t session_id) {
  yai_container_record_t record;
  int64_t now = (int64_t)time(NULL);

  if (!container_id || container_id[0] == '\0' || session_id == 0) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  if (!session_bound_to(&record, session_id)) {
    return -1;
  }

  record.session_domain.bound = 0;
  record.session_domain.active_session_id = 0;
  record.session_domain.session_mode = YAI_CONTAINER_SESSION_MODE_GLOBAL;
  record.session_domain.capability_mask = 0;
  record.session_domain.interactive_flags = 0;
  record.session_domain.escape_policy_class = YAI_CONTAINER_ESCAPE_NONE;
  record.session_domain.privileged_access = 0;
  record.session_domain.root_handle = 0;
  record.session_domain.runtime_view_handle = 0;
  record.state.session_status = YAI_CONTAINER_SESSION_STATUS_NONE;
  record.state.updated_at = now;
  return yai_container_model_upsert(&record);
}

int yai_container_rebind_session(const char *container_id,
                                 uint64_t old_session_id,
                                 uint64_t new_session_id,
                                 yai_container_session_mode_t mode,
                                 uint64_t capability_mask,
                                 uint64_t interactive_flags) {
  if (yai_container_unbind_session(container_id, old_session_id) != 0) {
    return -1;
  }
  return yai_container_bind_session(container_id, new_session_id, mode, capability_mask, interactive_flags);
}

int yai_container_session_enter(const char *container_id,
                                uint64_t session_id,
                                yai_container_bound_session_t *out_session) {
  yai_container_record_t record;
  yai_container_path_context_t context;

  if (!container_id || !out_session || session_id == 0) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  if (!session_bound_to(&record, session_id)) {
    return -1;
  }
  if (yai_container_path_context_load(container_id, &context) != 0) {
    return -1;
  }

  memset(out_session, 0, sizeof(*out_session));
  out_session->session_id = session_id;
  (void)snprintf(out_session->bound_container_id, sizeof(out_session->bound_container_id), "%s", container_id);
  out_session->session_mode = record.session_domain.session_mode;
  out_session->root_handle = record.session_domain.root_handle;
  out_session->path_context = context;
  out_session->runtime_view_handle = record.session_domain.runtime_view_handle;
  out_session->capability_mask = record.session_domain.capability_mask;
  out_session->escape_policy_class = record.session_domain.escape_policy_class;
  out_session->interactive_flags = record.session_domain.interactive_flags;
  out_session->bound = 1;
  return 0;
}

int yai_container_session_leave(const char *container_id, uint64_t session_id) {
  return yai_container_unbind_session(container_id, session_id);
}

int yai_container_session_get_root(const char *container_id,
                                   uint64_t session_id,
                                   yai_container_root_t *out_root) {
  yai_container_record_t record;

  if (!container_id || !out_root || session_id == 0) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  if (!session_bound_to(&record, session_id)) {
    return -1;
  }
  *out_root = record.root;
  return 0;
}

int yai_container_session_get_path_context(const char *container_id,
                                           uint64_t session_id,
                                           yai_container_path_context_t *out_context) {
  yai_container_record_t record;

  if (!container_id || !out_context || session_id == 0) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  if (!session_bound_to(&record, session_id)) {
    return -1;
  }
  return yai_container_path_context_load(container_id, out_context);
}

int yai_container_session_get_runtime_view(const char *container_id,
                                           uint64_t session_id,
                                           yai_container_runtime_view_t *out_view) {
  yai_container_record_t record;

  if (!container_id || !out_view || session_id == 0) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  if (!session_bound_to(&record, session_id)) {
    return -1;
  }
  return yai_container_runtime_view_get(container_id, out_view);
}

int yai_container_session_can_escape(const char *container_id, uint64_t session_id) {
  yai_container_record_t record;

  if (!container_id || container_id[0] == '\0' || session_id == 0) {
    return 0;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return 0;
  }
  if (!session_bound_to(&record, session_id)) {
    return 0;
  }
  return record.session_domain.escape_policy_class != YAI_CONTAINER_ESCAPE_NONE ? 1 : 0;
}

int yai_container_session_request_escape(const char *container_id,
                                         uint64_t session_id,
                                         yai_container_escape_policy_class_t requested_class) {
  yai_container_record_t record;
  int64_t now = (int64_t)time(NULL);

  if (!container_id || container_id[0] == '\0' || session_id == 0) {
    return -1;
  }
  if (requested_class > YAI_CONTAINER_ESCAPE_DEBUG) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  if (!session_bound_to(&record, session_id)) {
    return -1;
  }
  if (record.session_domain.escape_policy_class < requested_class) {
    return -1;
  }

  record.session_domain.interactive_flags |= (1ull << 0);
  record.state.updated_at = now;
  return yai_container_model_upsert(&record);
}

int yai_container_session_enter_recovery(const char *container_id,
                                         uint64_t session_id,
                                         uint64_t reason_flags) {
  yai_container_record_t record;

  if (!container_id || container_id[0] == '\0' || session_id == 0) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  if (!session_bound_to(&record, session_id)) {
    return -1;
  }
  if (record.session_domain.session_mode != YAI_CONTAINER_SESSION_MODE_RECOVERY &&
      record.session_domain.session_mode != YAI_CONTAINER_SESSION_MODE_PRIVILEGED) {
    return -1;
  }

  return yai_container_enter_recovery(container_id, reason_flags);
}

/* Compatibility shim used by previous C-2 surface. */
int yai_container_session_bind(const char *container_id, uint64_t session_id) {
  return yai_container_bind_session(container_id,
                                    session_id,
                                    YAI_CONTAINER_SESSION_MODE_NORMAL,
                                    UINT64_MAX,
                                    0);
}
