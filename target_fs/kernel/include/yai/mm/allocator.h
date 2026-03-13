#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uint64_t yai_mm_alloc_handle_t;

typedef enum {
  YAI_MM_ALLOC_CLASS_NONE = 0,
  YAI_MM_ALLOC_CLASS_KERNEL,
  YAI_MM_ALLOC_CLASS_RUNTIME,
  YAI_MM_ALLOC_CLASS_CONTAINER,
  YAI_MM_ALLOC_CLASS_SESSION,
  YAI_MM_ALLOC_CLASS_TRACE,
} yai_mm_alloc_class_t;

typedef enum {
  YAI_MM_ALLOC_FLAG_ZERO        = 1ull << 0,
  YAI_MM_ALLOC_FLAG_PINNED      = 1ull << 1,
  YAI_MM_ALLOC_FLAG_PERSISTENT  = 1ull << 2,
  YAI_MM_ALLOC_FLAG_ACCOUNTED   = 1ull << 3,
} yai_mm_alloc_flags_t;

typedef struct {
  yai_mm_alloc_handle_t handle;
  yai_mm_alloc_class_t alloc_class;
  uint64_t flags;
  size_t requested_size;
  size_t granted_size;
  uint64_t owner_handle;
  int64_t created_at;
} yai_mm_alloc_record_t;

void yai_mm_allocator_bootstrap(void);
void *yai_mm_alloc(size_t size, yai_mm_alloc_class_t alloc_class, uint64_t flags);
void yai_mm_free(void *ptr);
int yai_mm_alloc_record(void *ptr, yai_mm_alloc_record_t *out_record);

#ifdef __cplusplus
}
#endif
