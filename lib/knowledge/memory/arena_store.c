/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/knowledge/memory.h>

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

int yai_mind_arena_init(yai_mind_arena_t *arena, size_t size)
{
  if (!arena || size == 0) return YAI_MIND_ERR_INVALID_ARG;
  memset(arena, 0, sizeof(*arena));
  arena->buffer = (unsigned char *)malloc(size);
  if (!arena->buffer) return YAI_MIND_ERR_NO_MEMORY;
  arena->size = size;
  arena->offset = 0;
  return YAI_MIND_OK;
}

void *yai_mind_arena_alloc(yai_mind_arena_t *arena, size_t size, size_t alignment)
{
  size_t mask;
  size_t aligned_offset;

  if (!arena || !arena->buffer || size == 0) return NULL;
  if (alignment == 0) alignment = sizeof(void *);
  if ((alignment & (alignment - 1U)) != 0U) return NULL;

  mask = alignment - 1U;
  aligned_offset = (arena->offset + mask) & ~mask;

  if (aligned_offset > arena->size || size > (arena->size - aligned_offset)) return NULL;

  arena->offset = aligned_offset + size;
  return (void *)(arena->buffer + aligned_offset);
}

void yai_mind_arena_reset(yai_mind_arena_t *arena)
{
  if (!arena) return;
  arena->offset = 0;
}

void yai_mind_arena_destroy(yai_mind_arena_t *arena)
{
  if (!arena) return;
  free(arena->buffer);
  memset(arena, 0, sizeof(*arena));
}
