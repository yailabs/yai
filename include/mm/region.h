#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uint64_t yai_mm_region_handle_t;

typedef enum {
  YAI_MM_REGION_NONE = 0,
  YAI_MM_REGION_CODE,
  YAI_MM_REGION_HEAP,
  YAI_MM_REGION_STACK,
  YAI_MM_REGION_MMAP,
  YAI_MM_REGION_SHARED,
  YAI_MM_REGION_TRACE,
  YAI_MM_REGION_STATE,
} yai_mm_region_class_t;

typedef enum {
  YAI_MM_REGION_READ    = 1ull << 0,
  YAI_MM_REGION_WRITE   = 1ull << 1,
  YAI_MM_REGION_EXEC    = 1ull << 2,
  YAI_MM_REGION_SHARED_F = 1ull << 3,
  YAI_MM_REGION_LOCKED  = 1ull << 4,
} yai_mm_region_flags_t;

typedef struct {
  yai_mm_region_handle_t handle;
  yai_mm_region_class_t region_class;
  uintptr_t base;
  size_t length;
  uint64_t flags;
  uint64_t backing_handle;
} yai_mm_region_t;

void yai_mm_region_defaults(yai_mm_region_t *region);
int yai_mm_region_create(yai_mm_region_class_t region_class,
                         uintptr_t base,
                         size_t length,
                         uint64_t flags,
                         yai_mm_region_t *out_region);
int yai_mm_region_validate(const yai_mm_region_t *region);

#ifdef __cplusplus
}
#endif
