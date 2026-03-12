/* SPDX-License-Identifier: Apache-2.0 */
// src/model/registry/registry_query.c
//
// Registry query helpers (indexes) built on top of yai_law_registry()
// which is published by registry_load.c.

#define _POSIX_C_SOURCE 200809L

#include "yai/sdk/registry/registry_registry.h"

#include <stdlib.h>
#include <string.h>
#include <stddef.h>
#include <stdint.h>

typedef struct idx_entry {
  const char* key; // command id
  const yai_law_command_t* cmd;
} idx_entry_t;

typedef struct group_entry {
  const char* group;
  const yai_law_command_t** items;
  size_t len;     // count
  size_t cap;     // allocated
} group_entry_t;

static int g_inited = 0;

static idx_entry_t* g_id_index = NULL;
static size_t g_id_index_len = 0;

static group_entry_t* g_groups = NULL;
static size_t g_groups_len = 0;

static int cmp_id_entry(const void* a, const void* b) {
  const idx_entry_t* x = (const idx_entry_t*)a;
  const idx_entry_t* y = (const idx_entry_t*)b;
  if (!x->key && !y->key) return 0;
  if (!x->key) return -1;
  if (!y->key) return 1;
  return strcmp(x->key, y->key);
}

static int cmp_cstr(const void* a, const void* b) {
  const char* const* x = (const char* const*)a;
  const char* const* y = (const char* const*)b;
  if (!*x && !*y) return 0;
  if (!*x) return -1;
  if (!*y) return 1;
  return strcmp(*x, *y);
}

static const idx_entry_t* bsearch_id(const char* id) {
  if (!g_id_index || g_id_index_len == 0) return NULL;
  idx_entry_t needle = { .key = id, .cmd = NULL };
  return (const idx_entry_t*)bsearch(&needle, g_id_index, g_id_index_len, sizeof(idx_entry_t), cmp_id_entry);
}

static int find_group_slot(const char* group) {
  if (!group) return -1;
  for (size_t i = 0; i < g_groups_len; i++) {
    if (g_groups[i].group && strcmp(g_groups[i].group, group) == 0) return (int)i;
  }
  return -1;
}

static void free_groups(void) {
  if (!g_groups) return;
  for (size_t i = 0; i < g_groups_len; i++) {
    free((void*)g_groups[i].items);
    g_groups[i].items = NULL;
    g_groups[i].len = 0;
    g_groups[i].cap = 0;
  }
  free(g_groups);
  g_groups = NULL;
  g_groups_len = 0;
}

static void free_indexes(void) {
  free(g_id_index);
  g_id_index = NULL;
  g_id_index_len = 0;
  free_groups();
}

static int ensure_group_items_capacity(group_entry_t* ge, size_t need) {
  if (!ge) return 1;
  if (ge->cap >= need) return 0;

  size_t new_cap = ge->cap ? ge->cap : 8;
  while (new_cap < need) new_cap *= 2;

  const yai_law_command_t** nitems =
      (const yai_law_command_t**)realloc((void*)ge->items, new_cap * sizeof(yai_law_command_t*));
  if (!nitems) return 2;

  ge->items = nitems;
  ge->cap = new_cap;
  return 0;
}

/*
 * Build indexes once, using the published registry.
 * This assumes registry_load.c already knows how to init/cache the registry.
 */
static int registry_query_init(void) {
  if (g_inited) return 0;

  /* ensure registry is available */
  if (yai_law_registry_init() != 0) return 1;

  const yai_law_registry_t* r = yai_law_registry();
  if (!r) return 2;

  /* build id index */
  g_id_index_len = r->commands_len;
  if (g_id_index_len > 0) {
    g_id_index = (idx_entry_t*)calloc(g_id_index_len, sizeof(idx_entry_t));
    if (!g_id_index) return 3;

    for (size_t i = 0; i < g_id_index_len; i++) {
      g_id_index[i].key = r->commands[i].id;
      g_id_index[i].cmd = &r->commands[i];
    }

    qsort(g_id_index, g_id_index_len, sizeof(idx_entry_t), cmp_id_entry);
  }

  /* build group buckets */
  if (r->commands_len > 0) {
    const char** groups_tmp = (const char**)calloc(r->commands_len, sizeof(char*));
    if (!groups_tmp) {
      free_indexes();
      return 4;
    }

    size_t groups_tmp_len = 0;
    for (size_t i = 0; i < r->commands_len; i++) {
      groups_tmp[groups_tmp_len++] = r->commands[i].group;
    }

    qsort(groups_tmp, groups_tmp_len, sizeof(char*), cmp_cstr);

    /* count unique non-null groups */
    size_t unique = 0;
    const char* prev = NULL;
    for (size_t i = 0; i < groups_tmp_len; i++) {
      const char* g = groups_tmp[i];
      if (!g) continue;
      if (!prev || strcmp(prev, g) != 0) {
        unique++;
        prev = g;
      }
    }

    g_groups_len = unique;
    g_groups = (group_entry_t*)calloc(g_groups_len, sizeof(group_entry_t));
    if (!g_groups) {
      free(groups_tmp);
      free_indexes();
      return 5;
    }

    /* init group names */
    size_t gi = 0;
    prev = NULL;
    for (size_t i = 0; i < groups_tmp_len; i++) {
      const char* g = groups_tmp[i];
      if (!g) continue;
      if (!prev || strcmp(prev, g) != 0) {
        g_groups[gi].group = g;
        g_groups[gi].items = NULL;
        g_groups[gi].len = 0;
        g_groups[gi].cap = 0;
        gi++;
        prev = g;
      }
    }

    /* fill items (dynamic per group) */
    for (size_t i = 0; i < r->commands_len; i++) {
      const yai_law_command_t* c = &r->commands[i];
      if (!c->group) continue;

      int slot = find_group_slot(c->group);
      if (slot < 0) continue;

      group_entry_t* ge = &g_groups[(size_t)slot];
      if (ensure_group_items_capacity(ge, ge->len + 1) != 0) {
        free(groups_tmp);
        free_indexes();
        return 6;
      }
      ge->items[ge->len++] = c;
    }

    free(groups_tmp);
  }

  g_inited = 1;
  return 0;
}

/* ---- Public query API (declared in registry_registry.h) ---- */

const yai_law_command_t* yai_law_cmd_by_id(const char* id) {
  if (!id) return NULL;
  if (!g_inited) {
    if (registry_query_init() != 0) return NULL;
  }
  const idx_entry_t* e = bsearch_id(id);
  return e ? e->cmd : NULL;
}

yai_law_cmd_list_t yai_law_cmds_by_group(const char* group) {
  yai_law_cmd_list_t out = (yai_law_cmd_list_t){ .items = NULL, .len = 0 };

  if (!group) return out;
  if (!g_inited) {
    if (registry_query_init() != 0) return out;
  }

  for (size_t i = 0; i < g_groups_len; i++) {
    if (g_groups[i].group && strcmp(g_groups[i].group, group) == 0) {
      out.items = g_groups[i].items;
      out.len = g_groups[i].len;
      return out;
    }
  }

  return out;
}

int yai_law_command_has_output(const yai_law_command_t* c, const char* out) {
  if (!c || !out) return 0;
  for (size_t i = 0; i < c->outputs_len; i++) {
    if (c->outputs[i] && strcmp(c->outputs[i], out) == 0) return 1;
  }
  return 0;
}

int yai_law_command_has_side_effect(const yai_law_command_t* c, const char* eff) {
  if (!c || !eff) return 0;
  for (size_t i = 0; i < c->side_effects_len; i++) {
    if (c->side_effects[i] && strcmp(c->side_effects[i], eff) == 0) return 1;
  }
  return 0;
}