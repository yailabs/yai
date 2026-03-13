#pragma once

#ifndef YAI_DRIVERS_DRIVER_REGISTRY_H
#define YAI_DRIVERS_DRIVER_REGISTRY_H

#include <stddef.h>
#include <stdint.h>

#define YAI_DRIVER_NAME_MAX 63u
#define YAI_DRIVER_REGISTRY_MAX 64u

typedef enum {
  YAI_DRIVER_CLASS_NONE = 0,
  YAI_DRIVER_CLASS_CONSOLE,
  YAI_DRIVER_CLASS_INPUT,
  YAI_DRIVER_CLASS_STORAGE,
  YAI_DRIVER_CLASS_NETWORK,
  YAI_DRIVER_CLASS_BUS,
} yai_driver_class_t;

typedef struct {
  char name[YAI_DRIVER_NAME_MAX + 1u];
  yai_driver_class_t driver_class;
  uint64_t flags;
  uint8_t registered;
} yai_driver_descriptor_t;

int yai_driver_registry_register(const char *name,
                                 yai_driver_class_t driver_class,
                                 uint64_t flags);
int yai_driver_registry_lookup(const char *name,
                               yai_driver_descriptor_t *out_descriptor);
int yai_driver_registry_list(yai_driver_descriptor_t *entries,
                             size_t cap,
                             size_t *out_len);
void yai_driver_registry_reset(void);

#endif /* YAI_DRIVERS_DRIVER_REGISTRY_H */
