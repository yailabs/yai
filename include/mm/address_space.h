#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uint64_t yai_mm_address_space_handle_t;

typedef enum {
  YAI_MM_ADDRESS_SPACE_NONE = 0,
  YAI_MM_ADDRESS_SPACE_KERNEL,
  YAI_MM_ADDRESS_SPACE_RUNTIME,
  YAI_MM_ADDRESS_SPACE_CONTAINER,
  YAI_MM_ADDRESS_SPACE_SESSION,
} yai_mm_address_space_class_t;

typedef struct {
  yai_mm_address_space_handle_t handle;
  yai_mm_address_space_class_t space_class;
  uint64_t owner_handle;
  uint64_t region_count;
  uint64_t flags;
} yai_mm_address_space_t;

void yai_mm_address_space_defaults(yai_mm_address_space_t *space);
int yai_mm_address_space_create(yai_mm_address_space_class_t space_class,
                                uint64_t owner_handle,
                                yai_mm_address_space_t *out_space);
int yai_mm_address_space_attach_region(yai_mm_address_space_handle_t space_handle,
                                       uint64_t region_handle);

#ifdef __cplusplus
}
#endif
