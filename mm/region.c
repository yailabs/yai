#include <string.h>

#include <yai/mm/region.h>

static yai_mm_region_handle_t g_next_region_handle = 1u;

void yai_mm_region_defaults(yai_mm_region_t *region) {
  if (!region) {
    return;
  }
  memset(region, 0, sizeof(*region));
}

int yai_mm_region_validate(const yai_mm_region_t *region) {
  if (!region) {
    return -1;
  }
  if (region->region_class == YAI_MM_REGION_NONE) {
    return -1;
  }
  if (region->length == 0) {
    return -1;
  }
  return 0;
}

int yai_mm_region_create(yai_mm_region_class_t region_class,
                         uintptr_t base,
                         size_t length,
                         uint64_t flags,
                         yai_mm_region_t *out_region) {
  if (!out_region || region_class == YAI_MM_REGION_NONE || length == 0) {
    return -1;
  }

  yai_mm_region_defaults(out_region);
  out_region->handle = g_next_region_handle++;
  out_region->region_class = region_class;
  out_region->base = base;
  out_region->length = length;
  out_region->flags = flags;

  return yai_mm_region_validate(out_region);
}
