#include <string.h>

#include <yai/proc/registry.h>

static yai_process_registry_t g_process_registry;

void yai_process_registry_bootstrap(void) {
  memset(&g_process_registry, 0, sizeof(g_process_registry));
}

int yai_process_registry_register(const yai_process_t *process) {
  if (!process || process->handle == 0) {
    return -1;
  }
  if (g_process_registry.len >= YAI_PROCESS_REGISTRY_MAX) {
    return -1;
  }

  g_process_registry.entries[g_process_registry.len++] = *process;
  return 0;
}

int yai_process_registry_get(yai_process_handle_t handle, yai_process_t *out_process) {
  size_t i;

  if (handle == 0 || !out_process) {
    return -1;
  }

  for (i = 0; i < g_process_registry.len; ++i) {
    if (g_process_registry.entries[i].handle == handle) {
      *out_process = g_process_registry.entries[i];
      return 0;
    }
  }

  return -1;
}

int yai_process_registry_list(yai_process_t *entries, size_t cap, size_t *out_len) {
  size_t i;
  size_t n;

  if (!entries || cap == 0) {
    return -1;
  }

  n = g_process_registry.len < cap ? g_process_registry.len : cap;
  for (i = 0; i < n; ++i) {
    entries[i] = g_process_registry.entries[i];
  }

  if (out_len) {
    *out_len = n;
  }

  return 0;
}
