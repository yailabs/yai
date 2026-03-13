#include <yai/drivers/driver_registry.h>

#include <string.h>

#include "kernel/lib/internal/static_string.h"

static yai_driver_descriptor_t g_driver_registry[YAI_DRIVER_REGISTRY_MAX];

void yai_driver_registry_reset(void) {
  memset(g_driver_registry, 0, sizeof(g_driver_registry));
}

int yai_driver_registry_register(const char *name,
                                 yai_driver_class_t driver_class,
                                 uint64_t flags) {
  size_t i;

  if (!name || name[0] == '\0') {
    return -1;
  }

  for (i = 0; i < YAI_DRIVER_REGISTRY_MAX; ++i) {
    if (g_driver_registry[i].registered &&
        strncmp(g_driver_registry[i].name, name, YAI_DRIVER_NAME_MAX) == 0) {
      g_driver_registry[i].driver_class = driver_class;
      g_driver_registry[i].flags = flags;
      return 0;
    }
  }

  for (i = 0; i < YAI_DRIVER_REGISTRY_MAX; ++i) {
    if (!g_driver_registry[i].registered) {
      (void)yai_kernel_static_strlcpy(g_driver_registry[i].name, name, sizeof(g_driver_registry[i].name));
      g_driver_registry[i].driver_class = driver_class;
      g_driver_registry[i].flags = flags;
      g_driver_registry[i].registered = 1u;
      return 0;
    }
  }

  return -1;
}

int yai_driver_registry_lookup(const char *name,
                               yai_driver_descriptor_t *out_descriptor) {
  size_t i;

  if (!name || !out_descriptor) {
    return -1;
  }

  for (i = 0; i < YAI_DRIVER_REGISTRY_MAX; ++i) {
    if (g_driver_registry[i].registered &&
        strncmp(g_driver_registry[i].name, name, YAI_DRIVER_NAME_MAX) == 0) {
      *out_descriptor = g_driver_registry[i];
      return 0;
    }
  }

  return -1;
}

int yai_driver_registry_list(yai_driver_descriptor_t *entries,
                             size_t cap,
                             size_t *out_len) {
  size_t i;
  size_t written = 0;

  if (!entries || cap == 0 || !out_len) {
    return -1;
  }

  for (i = 0; i < YAI_DRIVER_REGISTRY_MAX && written < cap; ++i) {
    if (!g_driver_registry[i].registered) {
      continue;
    }
    entries[written++] = g_driver_registry[i];
  }

  *out_len = written;
  return 0;
}
