#define _POSIX_C_SOURCE 200809L

#include <stdio.h>
#include <string.h>
#include <time.h>

#include "internal/model.h"
#include "yai/container/registry.h"
#include "yai/container/services.h"

#define YAI_CONTAINER_SERVICE_MAX 64u

typedef struct {
  yai_container_service_entry_t entries[YAI_CONTAINER_SERVICE_MAX];
  size_t len;
} yai_container_services_file_t;

static int services_file_path(const char *container_id, char *out, size_t out_cap) {
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
  if (snprintf(out, out_cap, "%s/services.v1.bin", record_path) >= (int)out_cap) {
    return -1;
  }

  return 0;
}

static int services_load(const char *container_id, yai_container_services_file_t *file) {
  char path[4096];
  FILE *fp;

  if (!file) {
    return -1;
  }
  memset(file, 0, sizeof(*file));

  if (services_file_path(container_id, path, sizeof(path)) != 0) {
    return -1;
  }

  fp = fopen(path, "rb");
  if (!fp) {
    return 0;
  }

  if (fread(file, sizeof(*file), 1, fp) != 1) {
    fclose(fp);
    return -1;
  }
  fclose(fp);

  if (file->len > YAI_CONTAINER_SERVICE_MAX) {
    file->len = YAI_CONTAINER_SERVICE_MAX;
  }

  return 0;
}

static int services_save(const char *container_id, const yai_container_services_file_t *file) {
  char path[4096];
  FILE *fp;

  if (!file) {
    return -1;
  }

  if (services_file_path(container_id, path, sizeof(path)) != 0) {
    return -1;
  }

  fp = fopen(path, "wb");
  if (!fp) {
    return -1;
  }

  if (fwrite(file, sizeof(*file), 1, fp) != 1) {
    fclose(fp);
    return -1;
  }

  return fclose(fp) == 0 ? 0 : -1;
}

static int recalculate_service_state(const char *container_id, const yai_container_services_file_t *file) {
  yai_container_record_t record;
  size_t i;
  size_t ready = 0;
  size_t degraded = 0;

  if (!container_id || !file) {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }

  for (i = 0; i < file->len; ++i) {
    if (file->entries[i].status == YAI_CONTAINER_SERVICE_STATUS_READY) {
      ++ready;
    } else if (file->entries[i].status == YAI_CONTAINER_SERVICE_STATUS_DEGRADED) {
      ++degraded;
    }
  }

  record.state.services_online = (uint64_t)ready;
  if (file->len == 0) {
    record.state.service_status = YAI_CONTAINER_SERVICE_STATUS_NOT_INITIALIZED;
  } else if (degraded > 0) {
    record.state.service_status = YAI_CONTAINER_SERVICE_STATUS_DEGRADED;
  } else if (ready == file->len) {
    record.state.service_status = YAI_CONTAINER_SERVICE_STATUS_READY;
  } else {
    record.state.service_status = YAI_CONTAINER_SERVICE_STATUS_PARTIALLY_READY;
  }
  record.state.updated_at = (int64_t)time(NULL);

  return yai_container_registry_update(&record);
}

static int service_upsert_internal(const char *container_id,
                                   const yai_container_service_entry_t *entry) {
  yai_container_services_file_t file;
  size_t i;

  if (!container_id || !entry || container_id[0] == '\0' || entry->name[0] == '\0') {
    return -1;
  }

  if (!yai_container_model_exists(container_id)) {
    return -1;
  }

  if (services_load(container_id, &file) != 0) {
    return -1;
  }

  for (i = 0; i < file.len; ++i) {
    if (strncmp(file.entries[i].name, entry->name, YAI_CONTAINER_SERVICE_NAME_MAX) == 0) {
      file.entries[i] = *entry;
      if (services_save(container_id, &file) != 0) {
        return -1;
      }
      return recalculate_service_state(container_id, &file);
    }
  }

  if (file.len >= YAI_CONTAINER_SERVICE_MAX) {
    return -1;
  }

  file.entries[file.len++] = *entry;
  if (services_save(container_id, &file) != 0) {
    return -1;
  }
  return recalculate_service_state(container_id, &file);
}

int yai_container_service_register(const char *container_id,
                                   const char *name,
                                   uint64_t service_handle) {
  yai_container_service_entry_t entry;

  if (!container_id || !name || container_id[0] == '\0' || name[0] == '\0') {
    return -1;
  }

  memset(&entry, 0, sizeof(entry));
  if (snprintf(entry.name, sizeof(entry.name), "%s", name) >= (int)sizeof(entry.name)) {
    return -1;
  }
  entry.service_handle = service_handle == 0 ? (uint64_t)time(NULL) : service_handle;
  entry.status = YAI_CONTAINER_SERVICE_STATUS_NONE;
  entry.updated_at = (int64_t)time(NULL);

  return service_upsert_internal(container_id, &entry);
}

int yai_container_service_mark_ready(const char *container_id, const char *name) {
  yai_container_service_entry_t entry;

  if (yai_container_service_lookup(container_id, name, &entry) != 0) {
    return -1;
  }
  entry.status = YAI_CONTAINER_SERVICE_STATUS_READY;
  entry.updated_at = (int64_t)time(NULL);
  return service_upsert_internal(container_id, &entry);
}

int yai_container_service_mark_degraded(const char *container_id, const char *name) {
  yai_container_service_entry_t entry;

  if (yai_container_service_lookup(container_id, name, &entry) != 0) {
    return -1;
  }
  entry.status = YAI_CONTAINER_SERVICE_STATUS_DEGRADED;
  entry.updated_at = (int64_t)time(NULL);
  return service_upsert_internal(container_id, &entry);
}

int yai_container_service_lookup(const char *container_id,
                                 const char *name,
                                 yai_container_service_entry_t *out_entry) {
  yai_container_services_file_t file;
  size_t i;

  if (!container_id || !name || !out_entry || container_id[0] == '\0' || name[0] == '\0') {
    return -1;
  }
  if (!yai_container_model_exists(container_id)) {
    return -1;
  }
  if (services_load(container_id, &file) != 0) {
    return -1;
  }

  for (i = 0; i < file.len; ++i) {
    if (strncmp(file.entries[i].name, name, YAI_CONTAINER_SERVICE_NAME_MAX) == 0) {
      *out_entry = file.entries[i];
      return 0;
    }
  }
  return -1;
}

int yai_container_service_list(const char *container_id,
                               yai_container_service_entry_t *entries,
                               size_t cap,
                               size_t *out_len) {
  yai_container_services_file_t file;
  size_t i;

  if (!container_id || !entries || !out_len) {
    return -1;
  }
  if (!yai_container_model_exists(container_id)) {
    return -1;
  }
  if (services_load(container_id, &file) != 0) {
    return -1;
  }

  *out_len = (file.len < cap) ? file.len : cap;
  for (i = 0; i < *out_len; ++i) {
    entries[i] = file.entries[i];
  }
  return 0;
}

int yai_container_service_surface_get(const char *container_id,
                                      yai_container_service_surface_t *out_surface) {
  yai_container_services_file_t file;
  size_t i;

  if (!container_id || !out_surface || container_id[0] == '\0') {
    return -1;
  }
  if (!yai_container_model_exists(container_id)) {
    return -1;
  }
  if (services_load(container_id, &file) != 0) {
    return -1;
  }

  memset(out_surface, 0, sizeof(*out_surface));
  out_surface->service_surface_handle = (uint64_t)file.len;
  out_surface->total_services = file.len;
  if (out_surface->total_services > YAI_CONTAINER_SERVICE_SURFACE_MAX) {
    out_surface->total_services = YAI_CONTAINER_SERVICE_SURFACE_MAX;
  }
  for (i = 0; i < out_surface->total_services; ++i) {
    out_surface->entries[i] = file.entries[i];
    out_surface->service_surface_handle ^= file.entries[i].service_handle;
    if (file.entries[i].status == YAI_CONTAINER_SERVICE_STATUS_READY) {
      out_surface->ready_services += 1;
    } else if (file.entries[i].status == YAI_CONTAINER_SERVICE_STATUS_DEGRADED) {
      out_surface->degraded_services += 1;
    }
  }
  return 0;
}
