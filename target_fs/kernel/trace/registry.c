#include <stdio.h>
#include <string.h>

#include <yai/trace/event.h>
#include <yai/trace/registry.h>

static yai_trace_source_t g_trace_sources[YAI_TRACE_SOURCE_MAX];
static size_t g_trace_source_len = 0;
static uint64_t g_next_trace_source_handle = 1u;

void yai_trace_registry_bootstrap(void) {
  memset(g_trace_sources, 0, sizeof(g_trace_sources));
  g_trace_source_len = 0;
  g_next_trace_source_handle = 1u;
}

int yai_trace_source_register(yai_trace_domain_t domain,
                              uint64_t owner_handle,
                              const char *name,
                              yai_trace_source_t *out_source) {
  yai_trace_source_t src;

  if (domain == YAI_TRACE_DOMAIN_NONE || !name || name[0] == '\0') {
    return -1;
  }
  if (g_trace_source_len >= YAI_TRACE_SOURCE_MAX) {
    return -1;
  }

  memset(&src, 0, sizeof(src));
  src.source_handle = g_next_trace_source_handle++;
  src.owner_handle = owner_handle;
  src.domain = domain;

  if (snprintf(src.name, sizeof(src.name), "%s", name) >= (int)sizeof(src.name)) {
    return -1;
  }

  g_trace_sources[g_trace_source_len++] = src;

  if (out_source) {
    *out_source = src;
  }

  return 0;
}

int yai_trace_source_list(yai_trace_source_t *entries, size_t cap, size_t *out_len) {
  size_t i;
  size_t n;

  if (!entries || cap == 0) {
    return -1;
  }

  n = g_trace_source_len < cap ? g_trace_source_len : cap;
  for (i = 0; i < n; ++i) {
    entries[i] = g_trace_sources[i];
  }

  if (out_len) {
    *out_len = n;
  }

  return 0;
}
