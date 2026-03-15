#include <string.h>

#include <yai/mm/address_space.h>

static yai_mm_address_space_handle_t g_next_space_handle = 1u;

void yai_mm_address_space_defaults(yai_mm_address_space_t *space) {
  if (!space) {
    return;
  }
  memset(space, 0, sizeof(*space));
}

int yai_mm_address_space_create(yai_mm_address_space_class_t space_class,
                                uint64_t owner_handle,
                                yai_mm_address_space_t *out_space) {
  if (!out_space || space_class == YAI_MM_ADDRESS_SPACE_NONE) {
    return -1;
  }

  yai_mm_address_space_defaults(out_space);
  out_space->handle = g_next_space_handle++;
  out_space->space_class = space_class;
  out_space->owner_handle = owner_handle;
  return 0;
}

int yai_mm_address_space_attach_region(yai_mm_address_space_handle_t space_handle,
                                       uint64_t region_handle) {
  if (space_handle == 0 || region_handle == 0) {
    return -1;
  }

  return 0;
}
