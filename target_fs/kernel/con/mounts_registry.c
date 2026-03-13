#include <stdio.h>
#include <string.h>
#include <time.h>

#include "internal/model.h"
#include <yai/container/mounts.h>
#include <yai/container/registry.h>

#define YAI_CONTAINER_MOUNT_FILE_MAX 128u

typedef struct {
  yai_container_mount_t entries[YAI_CONTAINER_MOUNT_FILE_MAX];
  size_t len;
} yai_container_mount_file_t;

static int mounts_file_path(const char *container_id, char *out, size_t out_cap) {
  char record_path[4096];
  char *slash;

  if (yai_container_model_record_path(container_id, record_path, sizeof(record_path)) != 0) {
    return -1;
  }

  slash = strrchr(record_path, '/');
  if (!slash) {
    return -1;
  }

  *slash = '\0';
  if (snprintf(out, out_cap, "%s/mounts.v1.bin", record_path) >= (int)out_cap) {
    return -1;
  }

  return 0;
}

static int mount_target_valid(const char *target) {
  if (!target || target[0] == '\0') {
    return 0;
  }
  if (strncmp(target, "/mounts/", 8) != 0) {
    return 0;
  }
  if (strstr(target, "..") != NULL) {
    return 0;
  }
  return 1;
}

static int path_matches_target(const char *path, const char *target) {
  size_t tlen = 0;

  if (!path || !target) {
    return 0;
  }
  tlen = strlen(target);
  if (strncmp(path, target, tlen) != 0) {
    return 0;
  }
  return (path[tlen] == '\0' || path[tlen] == '/') ? 1 : 0;
}

int yai_container_attach_mount(const char *container_id,
                               const yai_container_mount_t *mount) {
  char path[4096];
  yai_container_mount_file_t file;
  yai_container_mount_t normalized;
  size_t i;
  FILE *fp;
  yai_container_record_t record;

  if (!container_id || !mount || container_id[0] == '\0' || !mount_target_valid(mount->target)) {
    return -1;
  }

  if (!yai_container_model_exists(container_id)) {
    return -1;
  }

  memset(&file, 0, sizeof(file));
  if (mounts_file_path(container_id, path, sizeof(path)) != 0) {
    return -1;
  }

  normalized = *mount;
  if (normalized.visibility_class == YAI_CONTAINER_VISIBILITY_INTERNAL &&
      normalized.mount_class != YAI_CONTAINER_MOUNT_INTERNAL_ROOT) {
    normalized.visibility_class = YAI_CONTAINER_VISIBILITY_ATTACHED;
  }
  if (normalized.attachability_class == YAI_CONTAINER_ATTACHABILITY_NONE) {
    normalized.attachability_class = YAI_CONTAINER_ATTACHABILITY_CONTROLLED;
  }

  fp = fopen(path, "rb");
  if (fp) {
    if (fread(&file, sizeof(file), 1, fp) != 1) {
      fclose(fp);
      return -1;
    }
    fclose(fp);
  }

  for (i = 0; i < file.len; ++i) {
    if (strcmp(file.entries[i].target, normalized.target) == 0) {
      file.entries[i] = normalized;
      file.entries[i].mount_id = (uint64_t)(i + 1u);
      fp = fopen(path, "wb");
      if (!fp) {
        return -1;
      }
      if (fwrite(&file, sizeof(file), 1, fp) != 1) {
        fclose(fp);
        return -1;
      }
      if (fclose(fp) != 0) {
        return -1;
      }
      if (yai_container_registry_get_record(container_id, &record) != 0) {
        return -1;
      }
      record.state.mount_status = YAI_CONTAINER_MOUNT_STATUS_APPLIED;
      record.state.updated_at = (int64_t)time(NULL);
      return yai_container_model_upsert(&record);
    }
  }

  if (file.len >= YAI_CONTAINER_MOUNT_FILE_MAX) {
    return -1;
  }

  normalized.mount_id = (uint64_t)(file.len + 1u);
  file.entries[file.len++] = normalized;

  fp = fopen(path, "wb");
  if (!fp) {
    return -1;
  }

  if (fwrite(&file, sizeof(file), 1, fp) != 1) {
    fclose(fp);
    return -1;
  }

  if (fclose(fp) != 0) {
    return -1;
  }
  if (yai_container_registry_get_record(container_id, &record) != 0) {
    return -1;
  }
  record.state.mount_status = YAI_CONTAINER_MOUNT_STATUS_APPLIED;
  record.state.updated_at = (int64_t)time(NULL);
  return yai_container_model_upsert(&record);
}

int yai_container_mounts_add(const char *container_id,
                             const yai_container_mount_t *mount) {
  return yai_container_attach_mount(container_id, mount);
}

int yai_container_get_mount_set(const char *container_id,
                                yai_container_mount_set_t *out_set) {
  yai_container_mount_file_t file;
  size_t i;

  if (!container_id || !out_set || container_id[0] == '\0') {
    return -1;
  }
  if (!yai_container_model_exists(container_id)) {
    return -1;
  }

  memset(&file, 0, sizeof(file));
  {
    char path[4096];
    FILE *fp;
    if (mounts_file_path(container_id, path, sizeof(path)) != 0) {
      return -1;
    }
    fp = fopen(path, "rb");
    if (fp) {
      if (fread(&file, sizeof(file), 1, fp) != 1) {
        fclose(fp);
        return -1;
      }
      fclose(fp);
    }
  }

  memset(out_set, 0, sizeof(*out_set));
  out_set->len = file.len > YAI_CONTAINER_MOUNT_MAX ? YAI_CONTAINER_MOUNT_MAX : file.len;
  for (i = 0; i < out_set->len; ++i) {
    out_set->entries[i] = file.entries[i];
  }
  return 0;
}

int yai_container_is_path_visible(const char *container_id,
                                  const char *container_path,
                                  int privileged_access) {
  yai_container_mount_set_t set;
  size_t i;

  if (!container_id || !container_path || container_id[0] == '\0' || container_path[0] != '/') {
    return -1;
  }

  if (strncmp(container_path, "/mounts/", 8) != 0) {
    return 1;
  }

  if (yai_container_get_mount_set(container_id, &set) != 0) {
    return 0;
  }

  for (i = 0; i < set.len; ++i) {
    const yai_container_mount_t *m = &set.entries[i];
    if (!path_matches_target(container_path, m->target)) {
      continue;
    }
    if (m->visibility_class == YAI_CONTAINER_VISIBILITY_HIDDEN) {
      return 0;
    }
    if (m->visibility_class == YAI_CONTAINER_VISIBILITY_PRIVILEGED_ONLY && !privileged_access) {
      return 0;
    }
    return 1;
  }

  return 0;
}
