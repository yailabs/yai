#include <time.h>

#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>

#include "../internal/model.h"
#include <yai/con/tree.h>

#ifndef YAI_CONTAINER_ZONE_COUNT
#define YAI_CONTAINER_ZONE_COUNT 8
#endif

static int ensure_dir(const char *path) {
  struct stat st;

  if (!path || path[0] == '\0') {
    return -1;
  }

  if (stat(path, &st) == 0) {
    return S_ISDIR(st.st_mode) ? 0 : -1;
  }

  return mkdir(path, 0755) == 0 ? 0 : -1;
}

static int container_dir_path(const char *container_id, char *out, size_t out_cap) {
  char record_path[4096];
  char *slash = NULL;

  if (yai_container_model_record_path(container_id, record_path, sizeof(record_path)) != 0) {
    return -1;
  }

  slash = strrchr(record_path, '/');
  if (!slash) {
    return -1;
  }
  *slash = '\0';
  return snprintf(out, out_cap, "%s", record_path) < (int)out_cap ? 0 : -1;
}

static int project_operational_tree(const char *projected_root) {
  static const char *zones[YAI_CONTAINER_ZONE_COUNT] = {
    "system", "state", "data", "mounts", "runtime", "sessions", "logs", "tmp"
  };
  size_t i;
  char zone_path[4096];

  if (ensure_dir(projected_root) != 0) {
    return -1;
  }

  for (i = 0; i < YAI_CONTAINER_ZONE_COUNT; ++i) {
    if (snprintf(zone_path, sizeof(zone_path), "%s/%s", projected_root, zones[i]) >= (int)sizeof(zone_path)) {
      return -1;
    }
    if (ensure_dir(zone_path) != 0) {
      return -1;
    }
  }

  return 0;
}

int yai_container_project_root(const char *container_id,
                               const char *backing_store_path,
                               yai_container_root_t *out_root) {
  char cdir[4096];
  yai_container_root_t root;

  if (!container_id || container_id[0] == '\0' || !out_root) {
    return -1;
  }

  if (container_dir_path(container_id, cdir, sizeof(cdir)) != 0) {
    return -1;
  }

  memset(&root, 0, sizeof(root));
  root.projection_ready = 1;
  root.container_root_handle = (uint64_t)time(NULL);
  root.root_projection_handle = root.container_root_handle;
  root.mount_view_handle = root.container_root_handle;
  root.attach_view_handle = root.container_root_handle;
  root.backing_store_handle = root.container_root_handle;

  if (snprintf(root.root_path, sizeof(root.root_path), "/") >= (int)sizeof(root.root_path)) {
    return -1;
  }
  if (snprintf(root.projected_root_host_path,
               sizeof(root.projected_root_host_path),
               "%s/projected-root",
               cdir) >= (int)sizeof(root.projected_root_host_path)) {
    return -1;
  }

  if (backing_store_path && backing_store_path[0]) {
    if (snprintf(root.backing_store_path,
                 sizeof(root.backing_store_path),
                 "%s",
                 backing_store_path) >= (int)sizeof(root.backing_store_path)) {
      return -1;
    }
  } else {
    if (snprintf(root.backing_store_path,
                 sizeof(root.backing_store_path),
                 "%s/backing-store",
                 cdir) >= (int)sizeof(root.backing_store_path)) {
      return -1;
    }
  }

  if (ensure_dir(root.backing_store_path) != 0) {
    return -1;
  }
  if (project_operational_tree(root.projected_root_host_path) != 0) {
    return -1;
  }

  *out_root = root;
  return 0;
}

int yai_container_root_projection_initialize(const char *container_id) {
  yai_container_record_t record;
  int64_t now = (int64_t)time(NULL);
  yai_container_root_t projected;

  if (!container_id || container_id[0] == '\0') {
    return -1;
  }

  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }

  if (yai_container_project_root(container_id, NULL, &projected) != 0) {
    return -1;
  }
  record.root = projected;
  record.root.container_root_handle = record.identity.state_handle;
  if (record.root.root_projection_handle == 0) {
    record.root.root_projection_handle = record.identity.state_handle;
  }
  if (record.root.backing_store_handle == 0) {
    record.root.backing_store_handle = record.identity.state_handle;
  }
  record.state.updated_at = now;
  record.state.root_status = YAI_CONTAINER_ROOT_STATUS_PROJECTED;
  if (record.state.mount_status == YAI_CONTAINER_MOUNT_STATUS_NONE) {
    record.state.mount_status = YAI_CONTAINER_MOUNT_STATUS_APPLIED;
  }
  record.state.runtime_state = (record.lifecycle.current == YAI_CONTAINER_LIFECYCLE_ACTIVE ||
                                record.lifecycle.current == YAI_CONTAINER_LIFECYCLE_OPEN)
                                   ? YAI_CONTAINER_RUNTIME_STATE_READY
                                   : YAI_CONTAINER_RUNTIME_STATE_BOOTSTRAPPING;
  if (record.state.health_state == YAI_CONTAINER_HEALTH_WARNING ||
      record.state.health_state == YAI_CONTAINER_HEALTH_DEGRADED) {
    record.state.health_state = YAI_CONTAINER_HEALTH_NOMINAL;
  }
  (void)yai_container_tree_project_defaults(&record.root, &record.tree);

  return yai_container_model_upsert(&record);
}
