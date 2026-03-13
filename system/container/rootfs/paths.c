#define _POSIX_C_SOURCE 200809L

#include <stddef.h>
#include <stdio.h>
#include <string.h>

#include "../runtime/internal/model.h"
#include "yai/container/paths.h"

static int append_segment(char *out, size_t out_cap, size_t *io_len, const char *seg) {
  int n = 0;

  if (!out || !io_len || !seg || seg[0] == '\0') {
    return -1;
  }

  if (*io_len > 0 && out[*io_len - 1] != '/') {
    if (*io_len + 1 >= out_cap) {
      return -1;
    }
    out[*io_len] = '/';
    *io_len += 1;
    out[*io_len] = '\0';
  }

  n = snprintf(out + *io_len, out_cap - *io_len, "%s", seg);
  if (n < 0 || (size_t)n >= out_cap - *io_len) {
    return -1;
  }
  *io_len += (size_t)n;
  return 0;
}

static int normalize_container_path(const char *input, char *out, size_t out_cap) {
  char buf[1024];
  char *tok = NULL;
  char *save = NULL;
  char *segments[64];
  size_t seg_len = 0;
  size_t i = 0;
  size_t len = 0;

  if (!input || !out || out_cap == 0) {
    return -1;
  }
  if (snprintf(buf, sizeof(buf), "%s", input) >= (int)sizeof(buf)) {
    return -1;
  }

  tok = strtok_r(buf, "/", &save);
  while (tok) {
    if (strcmp(tok, ".") == 0 || tok[0] == '\0') {
      tok = strtok_r(NULL, "/", &save);
      continue;
    }
    if (strcmp(tok, "..") == 0) {
      if (seg_len == 0) {
        return -1;
      }
      seg_len--;
      tok = strtok_r(NULL, "/", &save);
      continue;
    }
    if (seg_len >= (sizeof(segments) / sizeof(segments[0]))) {
      return -1;
    }
    segments[seg_len++] = tok;
    tok = strtok_r(NULL, "/", &save);
  }

  out[0] = '\0';
  for (i = 0; i < seg_len; ++i) {
    if (append_segment(out, out_cap, &len, segments[i]) != 0) {
      return -1;
    }
  }

  return 0;
}

int yai_container_paths_join(const char *root,
                             const char *relative,
                             char *out,
                             size_t out_cap) {
  int n;

  if (!root || !relative || !out || out_cap == 0) {
    return -1;
  }

  n = snprintf(out, out_cap, "%s/%s", root, relative);
  if (n < 0 || (size_t)n >= out_cap) {
    return -1;
  }

  return 0;
}

int yai_container_path_context_load(const char *container_id,
                                    yai_container_path_context_t *out_context) {
  yai_container_record_t record;

  if (!container_id || !out_context || container_id[0] == '\0') {
    return -1;
  }
  if (yai_container_model_get(container_id, &record) != 0) {
    return -1;
  }
  if (!record.root.projection_ready || record.root.projected_root_host_path[0] == '\0') {
    return -1;
  }

  memset(out_context, 0, sizeof(*out_context));
  (void)snprintf(out_context->container_id, sizeof(out_context->container_id), "%s", container_id);
  (void)snprintf(out_context->projected_root,
                 sizeof(out_context->projected_root),
                 "%s",
                 record.root.projected_root_host_path);
  (void)snprintf(out_context->backing_store,
                 sizeof(out_context->backing_store),
                 "%s",
                 record.root.backing_store_path);
  out_context->root_handle = record.root.root_projection_handle;
  out_context->backing_store_handle = record.root.backing_store_handle;
  return 0;
}

int yai_container_resolve_path(const yai_container_path_context_t *context,
                               const char *container_path,
                               char *out_host_path,
                               size_t out_cap) {
  char normalized[1024];

  if (!context || !container_path || !out_host_path || out_cap == 0) {
    return -1;
  }
  if (context->projected_root[0] == '\0') {
    return -1;
  }
  if (normalize_container_path(container_path, normalized, sizeof(normalized)) != 0) {
    return -1;
  }

  if (normalized[0] == '\0') {
    return snprintf(out_host_path, out_cap, "%s", context->projected_root) < (int)out_cap ? 0 : -1;
  }
  return yai_container_paths_join(context->projected_root, normalized, out_host_path, out_cap);
}

int yai_container_can_traverse(const char *container_id, const char *container_path) {
  yai_container_path_context_t context;
  char resolved[2048];

  if (yai_container_path_context_load(container_id, &context) != 0) {
    return -1;
  }
  return yai_container_resolve_path(&context, container_path, resolved, sizeof(resolved));
}
