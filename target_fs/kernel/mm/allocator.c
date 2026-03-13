#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include <yai/mm/allocator.h>

typedef struct {
  void *ptr;
  yai_mm_alloc_record_t record;
} yai_mm_alloc_slot_t;

#define YAI_MM_ALLOC_SLOT_MAX 256u

static yai_mm_alloc_slot_t g_alloc_slots[YAI_MM_ALLOC_SLOT_MAX];
static uint64_t g_next_alloc_handle = 1u;

static int find_slot(void *ptr) {
  size_t i;
  if (!ptr) {
    return -1;
  }
  for (i = 0; i < YAI_MM_ALLOC_SLOT_MAX; ++i) {
    if (g_alloc_slots[i].ptr == ptr) {
      return (int)i;
    }
  }
  return -1;
}

static int find_free_slot(void) {
  size_t i;
  for (i = 0; i < YAI_MM_ALLOC_SLOT_MAX; ++i) {
    if (g_alloc_slots[i].ptr == NULL) {
      return (int)i;
    }
  }
  return -1;
}

void yai_mm_allocator_bootstrap(void) {
  memset(g_alloc_slots, 0, sizeof(g_alloc_slots));
  g_next_alloc_handle = 1u;
}

void *yai_mm_alloc(size_t size, yai_mm_alloc_class_t alloc_class, uint64_t flags) {
  void *ptr;
  int slot;
  yai_mm_alloc_record_t rec;

  if (size == 0) {
    return NULL;
  }

  ptr = malloc(size);
  if (!ptr) {
    return NULL;
  }

  if ((flags & YAI_MM_ALLOC_FLAG_ZERO) != 0) {
    memset(ptr, 0, size);
  }

  slot = find_free_slot();
  if (slot < 0) {
    free(ptr);
    return NULL;
  }

  memset(&rec, 0, sizeof(rec));
  rec.handle = g_next_alloc_handle++;
  rec.alloc_class = alloc_class;
  rec.flags = flags;
  rec.requested_size = size;
  rec.granted_size = size;
  rec.created_at = (int64_t)time(NULL);

  g_alloc_slots[slot].ptr = ptr;
  g_alloc_slots[slot].record = rec;
  return ptr;
}

void yai_mm_free(void *ptr) {
  int slot;

  if (!ptr) {
    return;
  }

  slot = find_slot(ptr);
  if (slot >= 0) {
    memset(&g_alloc_slots[slot], 0, sizeof(g_alloc_slots[slot]));
  }

  free(ptr);
}

int yai_mm_alloc_record(void *ptr, yai_mm_alloc_record_t *out_record) {
  int slot;

  if (!ptr || !out_record) {
    return -1;
  }

  slot = find_slot(ptr);
  if (slot < 0) {
    return -1;
  }

  *out_record = g_alloc_slots[slot].record;
  return 0;
}
