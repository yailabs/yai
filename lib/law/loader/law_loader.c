#define _POSIX_C_SOURCE 200809L

#include "../internal.h"

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

static int yai_law_path_exists(const char *path) {
  struct stat st;
  return (path && stat(path, &st) == 0) ? 1 : 0;
}

int yai_law_safe_snprintf(char *out, size_t out_cap, const char *fmt, ...) {
  va_list ap;
  int n;
  if (!out || out_cap == 0 || !fmt) return -1;
  va_start(ap, fmt);
  n = vsnprintf(out, out_cap, fmt, ap);
  va_end(ap);
  if (n < 0 || (size_t)n >= out_cap) return -1;
  return 0;
}

int yai_law_read_text_file(const char *path, char *out, size_t out_cap) {
  FILE *f;
  size_t n;
  if (!path || !out || out_cap < 2) return -1;
  f = fopen(path, "rb");
  if (!f) return -1;
  n = fread(out, 1, out_cap - 1, f);
  out[n] = '\0';
  fclose(f);
  return 0;
}

int yai_law_json_extract_string(const char *json, const char *key, char *out, size_t out_cap) {
  char needle[128];
  const char *p;
  const char *q;
  if (!json || !key || !out || out_cap == 0) return -1;
  if (yai_law_safe_snprintf(needle, sizeof(needle), "\"%s\"", key) != 0) return -1;
  p = strstr(json, needle);
  if (!p) return -1;
  p = strchr(p, ':');
  if (!p) return -1;
  p++;
  while (*p == ' ' || *p == '\t' || *p == '\n' || *p == '\r') p++;
  if (*p != '"') return -1;
  p++;
  q = strchr(p, '"');
  if (!q) return -1;
  {
    size_t len = (size_t)(q - p);
    if (len >= out_cap) len = out_cap - 1;
    memcpy(out, p, len);
    out[len] = '\0';
  }
  return 0;
}

int yai_law_json_contains(const char *json, const char *needle) {
  if (!json || !needle) return 0;
  return strstr(json, needle) != NULL;
}

static int yai_law_resolve_root(char *out, size_t out_cap) {
  const char *env = getenv("YAI_LAW_EMBED_ROOT");
  const char *candidates[] = {"embedded/law", "../yai/embedded/law", "../../yai/embedded/law"};
  size_t i;
  if (env && env[0] && yai_law_path_exists(env)) {
    return yai_law_safe_snprintf(out, out_cap, "%s", env);
  }
  for (i = 0; i < sizeof(candidates) / sizeof(candidates[0]); i++) {
    if (yai_law_path_exists(candidates[i])) {
      return yai_law_safe_snprintf(out, out_cap, "%s", candidates[i]);
    }
  }
  return -1;
}

int yai_law_manifest_load(yai_law_runtime_t *rt, char *err, size_t err_cap);
int yai_law_compatibility_check(yai_law_runtime_t *rt, char *err, size_t err_cap);

int yai_law_load_runtime(yai_law_runtime_t *out, char *err, size_t err_cap) {
  if (!out) return -1;
  memset(out, 0, sizeof(*out));

  if (yai_law_resolve_root(out->root, sizeof(out->root)) != 0) {
    if (err && err_cap) (void)yai_law_safe_snprintf(err, err_cap, "embedded law root not found");
    return -1;
  }

  if (yai_law_manifest_load(out, err, err_cap) != 0) return -1;
  if (yai_law_compatibility_check(out, err, err_cap) != 0) return -1;
  if (yai_law_overlay_loader_validate(out, err, err_cap) != 0) {
    if (err && err_cap && !err[0]) (void)yai_law_safe_snprintf(err, err_cap, "%s", "overlay_index_load_failed");
    return -1;
  }
  return 0;
}
