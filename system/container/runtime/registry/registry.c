#define _POSIX_C_SOURCE 200809L

#include <dirent.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>

#include "../internal/model.h"
#include "yai/container/grants_view.h"
#include "yai/container/policy_view.h"
#include "yai/container/services.h"

static int fill_registry_entry(const yai_container_record_t *record,
                               yai_container_registry_entry_t *out_entry) {
  yai_container_policy_view_t policy;
  yai_container_grants_view_t grants;
  yai_container_service_surface_t surface;

  if (!record || !out_entry) {
    return -1;
  }

  memset(out_entry, 0, sizeof(*out_entry));
  (void)snprintf(out_entry->container_id, sizeof(out_entry->container_id), "%s", record->identity.container_id);
  out_entry->container_class = record->identity.container_class;
  out_entry->lifecycle_state = record->lifecycle.current;
  out_entry->root_handle = record->root.container_root_handle;
  out_entry->runtime_view_handle = record->session_domain.runtime_view_handle != 0
                                       ? record->session_domain.runtime_view_handle
                                       : record->identity.state_handle;
  out_entry->health_state = record->state.health_state;
  out_entry->recovery_state = record->state.recovery_status;
  out_entry->bound_session_count = record->session_domain.bound_session_count;
  out_entry->attached_daemon_count = record->state.daemon_bindings;
  out_entry->created_at = record->lifecycle.created_at;
  out_entry->updated_at = record->state.updated_at;

  if (yai_container_policy_view_get(record->identity.container_id, &policy) == 0) {
    out_entry->policy_view_handle = policy.policy_view_handle;
  }
  if (yai_container_grants_view_get(record->identity.container_id, &grants) == 0) {
    out_entry->grants_view_handle = grants.grants_view_handle;
  }
  if (yai_container_service_surface_get(record->identity.container_id, &surface) == 0) {
    out_entry->service_surface_handle = surface.service_surface_handle;
  }

  return 0;
}

int yai_container_registry_register(const yai_container_record_t *record) {
  if (!record || record->identity.container_id[0] == '\0') {
    return -1;
  }

  if (yai_container_model_exists(record->identity.container_id)) {
    return -1;
  }

  return yai_container_model_upsert(record);
}

int yai_container_registry_update(const yai_container_record_t *record) {
  if (!record || record->identity.container_id[0] == '\0') {
    return -1;
  }
  if (!yai_container_model_exists(record->identity.container_id)) {
    return -1;
  }
  return yai_container_model_upsert(record);
}

int yai_container_registry_get_record(const char *container_id,
                                      yai_container_record_t *out_record) {
  if (!container_id || !out_record || container_id[0] == '\0') {
    return -1;
  }
  return yai_container_model_get(container_id, out_record);
}

int yai_container_registry_lookup(const char *container_id,
                                  yai_container_registry_entry_t *out_entry) {
  yai_container_record_t record;

  if (!container_id || !out_entry || container_id[0] == '\0') {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  return fill_registry_entry(&record, out_entry);
}

int yai_container_registry_list(yai_container_registry_entry_t *entries,
                                size_t cap,
                                size_t *out_len) {
  char base[4096];
  DIR *dir = NULL;
  struct dirent *de;
  size_t len = 0;

  if (!entries || !out_len || cap == 0) {
    return -1;
  }

  if (yai_container_model_base_dir(base, sizeof(base)) != 0) {
    *out_len = 0;
    return 0;
  }

  dir = opendir(base);
  if (!dir) {
    *out_len = 0;
    return 0;
  }

  while ((de = readdir(dir)) != NULL) {
    yai_container_record_t record;
    char entry_path[4096];
    struct stat st;

    if (strcmp(de->d_name, ".") == 0 || strcmp(de->d_name, "..") == 0) {
      continue;
    }
    if (snprintf(entry_path, sizeof(entry_path), "%s/%s", base, de->d_name) >= (int)sizeof(entry_path)) {
      continue;
    }
    if (stat(entry_path, &st) != 0 || !S_ISDIR(st.st_mode)) {
      continue;
    }
    if (yai_container_model_get(de->d_name, &record) != 0) {
      continue;
    }
    if (len < cap) {
      (void)fill_registry_entry(&record, &entries[len]);
      ++len;
    }
  }

  closedir(dir);
  *out_len = len;
  return 0;
}

int yai_container_registry_unregister(const char *container_id) {
  if (!container_id || container_id[0] == '\0') {
    return -1;
  }
  return yai_container_model_remove(container_id);
}

int yai_container_registry_set_lifecycle(const char *container_id,
                                         yai_container_lifecycle_state_t next_state,
                                         int64_t at) {
  yai_container_record_t record;

  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }

  if (!yai_container_lifecycle_transition_allowed(record.lifecycle.current, next_state)) {
    return -1;
  }

  record.lifecycle.previous = record.lifecycle.current;
  record.lifecycle.current = next_state;
  record.lifecycle.updated_at = at;
  record.state.lifecycle_state = next_state;
  record.state.updated_at = at;

  if (next_state == YAI_CONTAINER_LIFECYCLE_SEALED) {
    record.lifecycle.sealed_at = at;
  }
  if (next_state == YAI_CONTAINER_LIFECYCLE_DESTROYED) {
    record.lifecycle.destroyed_at = at;
  }

  return yai_container_registry_update(&record);
}
