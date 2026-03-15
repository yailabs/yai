#define _POSIX_C_SOURCE 200809L

#include <errno.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

#include "model.h"

#ifndef PATH_MAX
#define PATH_MAX 4096
#endif

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

  if (errno == ENOENT) {
    char parent[PATH_MAX];
    char *slash = NULL;

    if (snprintf(parent, sizeof(parent), "%s", path) >= (int)sizeof(parent)) {
      return -1;
    }

    slash = strrchr(parent, '/');
    if (!slash || slash == parent) {
      return -1;
    }

    *slash = '\0';
    if (ensure_dir(parent) != 0) {
      return -1;
    }

    return mkdir(path, 0755) == 0 ? 0 : -1;
  }

  return -1;
}

static int container_id_valid(const char *id) {
  size_t i = 0;

  if (!id || id[0] == '\0') {
    return 0;
  }

  for (i = 0; id[i] != '\0'; ++i) {
    unsigned char c = (unsigned char)id[i];
    if (!((c >= 'a' && c <= 'z') ||
          (c >= 'A' && c <= 'Z') ||
          (c >= '0' && c <= '9') ||
          c == '-' || c == '_')) {
      return 0;
    }
  }

  return 1;
}

int yai_container_model_base_dir(char *out, size_t out_cap) {
  const char *override = getenv("YAI_CONTAINER_HOME");
  const char *home = getenv("HOME");

  if (!out || out_cap == 0) {
    return -1;
  }

  if (override && override[0]) {
    if (snprintf(out, out_cap, "%s", override) >= (int)out_cap) {
      return -1;
    }
  } else {
    if (!home || !home[0]) {
      return -1;
    }
    if (snprintf(out, out_cap, "%s/.yai/containers", home) >= (int)out_cap) {
      return -1;
    }
  }

  return ensure_dir(out);
}

int yai_container_model_record_path(const char *container_id, char *out, size_t out_cap) {
  char base[PATH_MAX];
  char container_dir[PATH_MAX];

  if (!container_id_valid(container_id) || !out || out_cap == 0) {
    return -1;
  }

  if (yai_container_model_base_dir(base, sizeof(base)) != 0) {
    return -1;
  }

  if (snprintf(container_dir, sizeof(container_dir), "%s/%s", base, container_id) >= (int)sizeof(container_dir)) {
    return -1;
  }

  if (ensure_dir(container_dir) != 0) {
    return -1;
  }

  if (snprintf(out, out_cap, "%s/record.v1.bin", container_dir) >= (int)out_cap) {
    return -1;
  }

  return 0;
}

int yai_container_model_exists(const char *container_id) {
  char path[PATH_MAX];
  struct stat st;

  if (yai_container_model_record_path(container_id, path, sizeof(path)) != 0) {
    return 0;
  }

  if (stat(path, &st) != 0) {
    return 0;
  }

  return S_ISREG(st.st_mode) ? 1 : 0;
}

int yai_container_model_upsert(const yai_container_record_t *record) {
  char path[PATH_MAX];
  FILE *fp = NULL;

  if (!record || !container_id_valid(record->identity.container_id)) {
    return -1;
  }

  if (yai_container_model_record_path(record->identity.container_id, path, sizeof(path)) != 0) {
    return -1;
  }

  fp = fopen(path, "wb");
  if (!fp) {
    return -1;
  }

  if (fwrite(record, sizeof(*record), 1, fp) != 1) {
    fclose(fp);
    return -1;
  }

  if (fclose(fp) != 0) {
    return -1;
  }

  return 0;
}

int yai_container_model_get(const char *container_id, yai_container_record_t *out_record) {
  char path[PATH_MAX];
  FILE *fp = NULL;

  if (!out_record || !container_id_valid(container_id)) {
    return -1;
  }

  if (yai_container_model_record_path(container_id, path, sizeof(path)) != 0) {
    return -1;
  }

  fp = fopen(path, "rb");
  if (!fp) {
    return -1;
  }

  if (fread(out_record, sizeof(*out_record), 1, fp) != 1) {
    fclose(fp);
    return -1;
  }

  if (fclose(fp) != 0) {
    return -1;
  }

  return 0;
}

int yai_container_model_remove(const char *container_id) {
  char path[PATH_MAX];

  if (!container_id_valid(container_id)) {
    return -1;
  }

  if (yai_container_model_record_path(container_id, path, sizeof(path)) != 0) {
    return -1;
  }

  if (unlink(path) != 0 && errno != ENOENT) {
    return -1;
  }

  return 0;
}
