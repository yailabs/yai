/* SPDX-License-Identifier: Apache-2.0 */

#include "yai/sdk/catalog.h"
#include "yai/sdk/registry/registry_registry.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct group_counter {
  char group[32];
  size_t count;
  size_t write_cursor;
} group_counter_t;

static int cmp_group(const void *a, const void *b)
{
  const yai_sdk_command_group_t *ga = (const yai_sdk_command_group_t *)a;
  const yai_sdk_command_group_t *gb = (const yai_sdk_command_group_t *)b;
  return strcmp(ga->group, gb->group);
}

static int cmp_command_by_name(const void *a, const void *b)
{
  const yai_sdk_command_ref_t *ca = (const yai_sdk_command_ref_t *)a;
  const yai_sdk_command_ref_t *cb = (const yai_sdk_command_ref_t *)b;
  return strcmp(ca->name, cb->name);
}

static int cmp_command_ptr_canonical(const void *a, const void *b)
{
  const yai_sdk_command_ref_t *ca = *(const yai_sdk_command_ref_t * const *)a;
  const yai_sdk_command_ref_t *cb = *(const yai_sdk_command_ref_t * const *)b;
  int d;
  d = strcmp(ca->entrypoint, cb->entrypoint);
  if (d != 0) return d;
  d = strcmp(ca->topic, cb->topic);
  if (d != 0) return d;
  d = strcmp(ca->op, cb->op);
  if (d != 0) return d;
  d = strcmp(ca->canonical_path, cb->canonical_path);
  if (d != 0) return d;
  return strcmp(ca->id, cb->id);
}

static int cmp_entrypoint_cstr(const void *a, const void *b)
{
  const char * const *ea = (const char * const *)a;
  const char * const *eb = (const char * const *)b;
  return strcmp(*ea, *eb);
}

static int find_counter_slot(group_counter_t *counters, size_t len, const char *group)
{
  if (!counters || !group || !group[0]) return -1;
  for (size_t i = 0; i < len; i++) {
    if (strcmp(counters[i].group, group) == 0) return (int)i;
  }
  return -1;
}

static void build_fallback_path(char *dst, size_t dst_sz,
                                const char *entrypoint,
                                const char *topic,
                                const char *op)
{
  if (!dst || dst_sz == 0) return;
  dst[0] = '\0';

  if (entrypoint && entrypoint[0]) {
    if (topic && topic[0]) {
      if (op && op[0]) {
        snprintf(dst, dst_sz, "%s %s %s", entrypoint, topic, op);
      } else {
        snprintf(dst, dst_sz, "%s %s", entrypoint, topic);
      }
    } else if (op && op[0]) {
      snprintf(dst, dst_sz, "%s %s", entrypoint, op);
    } else {
      snprintf(dst, dst_sz, "%s", entrypoint);
    }
  }
}

static int surface_matches(const char *surface, int mask)
{
  if (!surface || !surface[0]) return (mask & YAI_SDK_CATALOG_SURFACE_PLUMBING) != 0;
  if (strcmp(surface, "surface") == 0 || strcmp(surface, "user") == 0) {
    return (mask & YAI_SDK_CATALOG_SURFACE_SURFACE) != 0;
  }
  if (strcmp(surface, "ancillary") == 0 || strcmp(surface, "tool") == 0) {
    return (mask & YAI_SDK_CATALOG_SURFACE_ANCILLARY) != 0;
  }
  if (strcmp(surface, "plumbing") == 0 || strcmp(surface, "internal") == 0) {
    return (mask & YAI_SDK_CATALOG_SURFACE_PLUMBING) != 0;
  }
  return 0;
}

static int stability_matches(const char *stability, int mask)
{
  if (!stability || !stability[0]) return (mask & YAI_SDK_CATALOG_STABILITY_PLANNED) != 0;
  if (strcmp(stability, "stable") == 0) return (mask & YAI_SDK_CATALOG_STABILITY_STABLE) != 0;
  if (strcmp(stability, "experimental") == 0 || strcmp(stability, "beta") == 0) {
    return (mask & YAI_SDK_CATALOG_STABILITY_EXPERIMENTAL) != 0;
  }
  if (strcmp(stability, "planned") == 0) return (mask & YAI_SDK_CATALOG_STABILITY_PLANNED) != 0;
  if (strcmp(stability, "deprecated") == 0) return (mask & YAI_SDK_CATALOG_STABILITY_DEPRECATED) != 0;
  return 0;
}

static int command_matches_filter(const yai_sdk_command_ref_t *c, const yai_sdk_catalog_filter_t *filter)
{
  int surface_mask = YAI_SDK_CATALOG_SURFACE_ALL;
  int stability_mask = YAI_SDK_CATALOG_STABILITY_ALL;
  int include_hidden = 1;
  int include_deprecated = 1;

  if (!c) return 0;
  if (filter) {
    if (filter->surface_mask != 0) surface_mask = filter->surface_mask;
    if (filter->stability_mask != 0) stability_mask = filter->stability_mask;
    include_hidden = filter->include_hidden ? 1 : 0;
    include_deprecated = filter->include_deprecated ? 1 : 0;

    if (filter->entrypoint && filter->entrypoint[0] && strcmp(c->entrypoint, filter->entrypoint) != 0) return 0;
    if (filter->topic && filter->topic[0] && strcmp(c->topic, filter->topic) != 0) return 0;
  }

  if (!surface_matches(c->surface, surface_mask)) return 0;
  if (!stability_matches(c->stability, stability_mask)) return 0;
  if (!include_hidden && c->hidden) return 0;
  if (!include_deprecated && c->deprecated) return 0;
  return 1;
}

static void free_partial(yai_sdk_command_catalog_t *out)
{
  if (!out) return;
  free(out->commands_sorted);
  out->commands_sorted = NULL;
  out->command_count = 0;

  if (!out->groups) return;
  for (size_t i = 0; i < out->group_count; i++) {
    free(out->groups[i].commands);
    out->groups[i].commands = NULL;
    out->groups[i].command_count = 0;
  }
  free(out->groups);
  out->groups = NULL;
  out->group_count = 0;
}

int yai_sdk_command_catalog_load(yai_sdk_command_catalog_t *out)
{
  const yai_law_registry_t *reg;
  group_counter_t *counters = NULL;
  size_t counter_len = 0;
  size_t total_commands = 0;

  if (!out) return 1;
  memset(out, 0, sizeof(*out));

  if (yai_law_registry_init() != 0) return 2;
  reg = yai_law_registry();
  if (!reg || !reg->commands || reg->commands_len == 0) return 3;

  counters = (group_counter_t *)calloc(reg->commands_len, sizeof(*counters));
  if (!counters) return 4;

  for (size_t i = 0; i < reg->commands_len; i++) {
    const yai_law_command_t *c = &reg->commands[i];
    const char *group = (c && c->group && c->group[0]) ? c->group : "legacy";
    int slot;

    if (!c || !c->name || !c->id) continue;
    total_commands++;

    slot = find_counter_slot(counters, counter_len, group);
    if (slot < 0) {
      slot = (int)counter_len++;
      snprintf(counters[(size_t)slot].group, sizeof(counters[(size_t)slot].group), "%s", group);
      counters[(size_t)slot].count = 0;
      counters[(size_t)slot].write_cursor = 0;
    }
    counters[(size_t)slot].count++;
  }

  if (counter_len == 0 || total_commands == 0) {
    free(counters);
    return 5;
  }

  out->groups = (yai_sdk_command_group_t *)calloc(counter_len, sizeof(*out->groups));
  if (!out->groups) {
    free(counters);
    return 6;
  }
  out->group_count = counter_len;

  for (size_t i = 0; i < counter_len; i++) {
    snprintf(out->groups[i].group, sizeof(out->groups[i].group), "%s", counters[i].group);
    out->groups[i].command_count = counters[i].count;
    out->groups[i].commands = (yai_sdk_command_ref_t *)calloc(counters[i].count, sizeof(yai_sdk_command_ref_t));
    if (!out->groups[i].commands) {
      free(counters);
      free_partial(out);
      return 7;
    }
  }

  for (size_t i = 0; i < reg->commands_len; i++) {
    const yai_law_command_t *c = &reg->commands[i];
    const char *group = (c && c->group && c->group[0]) ? c->group : "legacy";
    int cslot;
    size_t w;
    yai_sdk_command_ref_t *ref;

    if (!c || !c->name || !c->id) continue;
    cslot = find_counter_slot(counters, counter_len, group);
    if (cslot < 0) continue;

    w = counters[(size_t)cslot].write_cursor++;
    if (w >= out->groups[(size_t)cslot].command_count) {
      free(counters);
      free_partial(out);
      return 8;
    }

    ref = &out->groups[(size_t)cslot].commands[w];
    snprintf(ref->group, sizeof(ref->group), "%s", group);
    snprintf(ref->name, sizeof(ref->name), "%s", c->name);
    snprintf(ref->id, sizeof(ref->id), "%s", c->id);

    snprintf(ref->surface, sizeof(ref->surface), "%s",
             (c->surface && c->surface[0]) ? c->surface : "plumbing");
    snprintf(ref->entrypoint, sizeof(ref->entrypoint), "%s",
             (c->entrypoint && c->entrypoint[0]) ? c->entrypoint : "run");
    snprintf(ref->topic, sizeof(ref->topic), "%s",
             (c->topic && c->topic[0]) ? c->topic : group);
    snprintf(ref->op, sizeof(ref->op), "%s",
             (c->op && c->op[0]) ? c->op : c->name);
    snprintf(ref->domain, sizeof(ref->domain), "%s",
             (c->domain && c->domain[0]) ? c->domain : "internal");
    snprintf(ref->layer, sizeof(ref->layer), "%s",
             (c->layer && c->layer[0]) ? c->layer : "runtime");
    snprintf(ref->stability, sizeof(ref->stability), "%s",
             (c->stability && c->stability[0]) ? c->stability : "planned");

    if (c->canonical_path && c->canonical_path[0]) {
      snprintf(ref->canonical_path, sizeof(ref->canonical_path), "%s", c->canonical_path);
    } else {
      build_fallback_path(ref->canonical_path, sizeof(ref->canonical_path),
                          ref->entrypoint, ref->topic, ref->op);
    }

    ref->help_order = c->help_order;
    ref->hidden = c->hidden ? 1 : 0;
    ref->deprecated = c->deprecated ? 1 : 0;
    if (c->replaced_by && c->replaced_by[0]) {
      snprintf(ref->replaced_by, sizeof(ref->replaced_by), "%s", c->replaced_by);
    }

    ref->aliases = c->aliases;
    ref->aliases_len = c->aliases_len;
    ref->outputs = c->outputs;
    ref->outputs_len = c->outputs_len;
    ref->side_effects = c->side_effects;
    ref->side_effects_len = c->side_effects_len;

    if (c->summary && c->summary[0]) {
      snprintf(ref->summary, sizeof(ref->summary), "%s", c->summary);
    } else {
      snprintf(ref->summary, sizeof(ref->summary), "No description.");
    }
  }

  for (size_t i = 0; i < out->group_count; i++) {
    qsort(out->groups[i].commands,
          out->groups[i].command_count,
          sizeof(out->groups[i].commands[0]),
          cmp_command_by_name);
  }
  qsort(out->groups, out->group_count, sizeof(out->groups[0]), cmp_group);

  out->commands_sorted = (yai_sdk_command_ref_t **)calloc(total_commands, sizeof(yai_sdk_command_ref_t *));
  if (!out->commands_sorted) {
    free(counters);
    free_partial(out);
    return 9;
  }
  out->command_count = total_commands;

  {
    size_t k = 0;
    for (size_t i = 0; i < out->group_count; i++) {
      for (size_t j = 0; j < out->groups[i].command_count; j++) {
        out->commands_sorted[k++] = &out->groups[i].commands[j];
      }
    }
    qsort(out->commands_sorted,
          out->command_count,
          sizeof(out->commands_sorted[0]),
          cmp_command_ptr_canonical);
  }

  free(counters);
  return 0;
}

void yai_sdk_command_catalog_free(yai_sdk_command_catalog_t *cat)
{
  if (!cat) return;
  free_partial(cat);
}

const yai_sdk_command_group_t *yai_sdk_command_catalog_find_group(
    const yai_sdk_command_catalog_t *cat,
    const char *group)
{
  if (!cat || !group || !group[0]) return NULL;
  for (size_t i = 0; i < cat->group_count; i++) {
    if (strcmp(cat->groups[i].group, group) == 0) return &cat->groups[i];
  }
  return NULL;
}

const yai_sdk_command_ref_t *yai_sdk_command_catalog_find_command(
    const yai_sdk_command_catalog_t *cat,
    const char *group,
    const char *name)
{
  const yai_sdk_command_group_t *g = yai_sdk_command_catalog_find_group(cat, group);
  if (!g || !name || !name[0]) return NULL;
  for (size_t i = 0; i < g->command_count; i++) {
    if (strcmp(g->commands[i].name, name) == 0) return &g->commands[i];
  }
  return NULL;
}

const yai_sdk_command_ref_t *yai_sdk_command_catalog_find_by_id(
    const yai_sdk_command_catalog_t *cat,
    const char *canonical_id)
{
  if (!cat || !canonical_id || !canonical_id[0]) return NULL;
  for (size_t i = 0; i < cat->command_count; i++) {
    const yai_sdk_command_ref_t *c = cat->commands_sorted[i];
    if (strcmp(c->id, canonical_id) == 0) return c;
  }
  return NULL;
}

const yai_sdk_command_ref_t *yai_sdk_command_catalog_find_by_path(
    const yai_sdk_command_catalog_t *cat,
    const char *entrypoint,
    const char *topic,
    const char *op,
    int surface_mask)
{
  yai_sdk_catalog_filter_t f = {0};
  if (!cat || !entrypoint || !entrypoint[0]) return NULL;

  f.surface_mask = (surface_mask == 0) ? YAI_SDK_CATALOG_SURFACE_ALL : surface_mask;
  f.stability_mask = YAI_SDK_CATALOG_STABILITY_ALL;
  f.entrypoint = entrypoint;
  f.topic = topic;
  f.include_hidden = 1;
  f.include_deprecated = 1;

  for (size_t i = 0; i < cat->command_count; i++) {
    const yai_sdk_command_ref_t *c = cat->commands_sorted[i];
    if (!command_matches_filter(c, &f)) continue;
    if (op && op[0] && strcmp(c->op, op) != 0) continue;
    return c;
  }
  return NULL;
}

const yai_sdk_command_ref_t *yai_sdk_command_catalog_find_by_canonical_path(
    const yai_sdk_command_catalog_t *cat,
    const char *canonical_path,
    const yai_sdk_catalog_filter_t *filter)
{
  if (!cat || !canonical_path || !canonical_path[0]) return NULL;
  for (size_t i = 0; i < cat->command_count; i++) {
    const yai_sdk_command_ref_t *c = cat->commands_sorted[i];
    if (!command_matches_filter(c, filter)) continue;
    if (strcmp(c->canonical_path, canonical_path) == 0) return c;
  }
  return NULL;
}

const yai_sdk_command_ref_t *yai_sdk_command_catalog_find_by_alias(
    const yai_sdk_command_catalog_t *cat,
    const char *alias,
    int *ambiguous)
{
  const yai_sdk_command_ref_t *found = NULL;
  if (ambiguous) *ambiguous = 0;
  if (!cat || !alias || !alias[0]) return NULL;

  for (size_t i = 0; i < cat->command_count; i++) {
    const yai_sdk_command_ref_t *c = cat->commands_sorted[i];
    for (size_t j = 0; j < c->aliases_len; j++) {
      if (!c->aliases[j] || !c->aliases[j][0]) continue;
      if (strcmp(c->aliases[j], alias) == 0) {
        if (!found) {
          found = c;
        } else if (strcmp(found->id, c->id) != 0) {
          if (ambiguous) *ambiguous = 1;
          return NULL;
        }
      }
    }
  }
  return found;
}

const yai_sdk_command_ref_t *yai_sdk_command_catalog_resolve_alias(
    const yai_sdk_command_catalog_t *cat,
    const char *alias,
    yai_sdk_catalog_resolve_status_t *status)
{
  int ambiguous = 0;
  const yai_sdk_command_ref_t *c;

  if (status) *status = YAI_SDK_CATALOG_RESOLVE_BAD_ARGS;
  if (!cat || !alias || !alias[0]) return NULL;

  c = yai_sdk_command_catalog_find_by_alias(cat, alias, &ambiguous);
  if (c) {
    if (status) *status = YAI_SDK_CATALOG_RESOLVE_OK;
    return c;
  }

  if (ambiguous) {
    if (status) *status = YAI_SDK_CATALOG_RESOLVE_AMBIGUOUS_ALIAS;
  } else if (status) {
    *status = YAI_SDK_CATALOG_RESOLVE_NOT_FOUND;
  }
  return NULL;
}

size_t yai_sdk_command_catalog_query(
    const yai_sdk_command_catalog_t *cat,
    const yai_sdk_catalog_filter_t *filter,
    const yai_sdk_command_ref_t **out_matches,
    size_t out_cap)
{
  size_t n = 0;
  if (!cat) return 0;

  for (size_t i = 0; i < cat->command_count; i++) {
    const yai_sdk_command_ref_t *c = cat->commands_sorted[i];
    if (!command_matches_filter(c, filter)) continue;
    if (out_matches && n < out_cap) out_matches[n] = c;
    n++;
  }
  return n;
}

const yai_sdk_command_ref_t *yai_sdk_command_catalog_resolve_path(
    const yai_sdk_command_catalog_t *cat,
    const char **tokens,
    size_t token_count,
    const yai_sdk_catalog_filter_t *filter,
    yai_sdk_catalog_resolve_status_t *status)
{
  yai_sdk_catalog_filter_t local_filter;
  const yai_sdk_command_ref_t *found = NULL;
  int ambiguous_alias = 0;
  int seen_entrypoint = 0;
  int seen_topic = 0;
  char path_buf[192];

  if (status) *status = YAI_SDK_CATALOG_RESOLVE_BAD_ARGS;
  if (!cat || !tokens || token_count == 0 || !tokens[0] || !tokens[0][0]) return NULL;

  memset(&local_filter, 0, sizeof(local_filter));
  if (filter) local_filter = *filter;
  if (local_filter.surface_mask == 0) local_filter.surface_mask = YAI_SDK_CATALOG_SURFACE_ALL;
  if (local_filter.stability_mask == 0) local_filter.stability_mask = YAI_SDK_CATALOG_STABILITY_ALL;
  local_filter.entrypoint = tokens[0];

  if (token_count > 1 && tokens[1] && tokens[1][0]) {
    local_filter.topic = tokens[1];
  }

  for (size_t i = 0; i < cat->command_count; i++) {
    const yai_sdk_command_ref_t *c = cat->commands_sorted[i];
    if (!command_matches_filter(c, &local_filter)) continue;
    seen_entrypoint = 1;
    if (token_count > 1 && strcmp(c->topic, tokens[1]) == 0) seen_topic = 1;

    if (token_count == 1) {
      continue;
    }
    if (token_count == 2) {
      if (strcmp(c->topic, tokens[1]) == 0) {
        found = c;
        break;
      }
      continue;
    }

    if (strcmp(c->topic, tokens[1]) == 0 && strcmp(c->op, tokens[2]) == 0) {
      found = c;
      break;
    }
  }

  if (found) {
    if (status) *status = YAI_SDK_CATALOG_RESOLVE_OK;
    return found;
  }

  path_buf[0] = '\0';
  for (size_t i = 0; i < token_count && i < 3; i++) {
    if (!tokens[i] || !tokens[i][0]) continue;
    if (path_buf[0]) strncat(path_buf, " ", sizeof(path_buf) - strlen(path_buf) - 1);
    strncat(path_buf, tokens[i], sizeof(path_buf) - strlen(path_buf) - 1);
  }

  found = yai_sdk_command_catalog_find_by_canonical_path(cat, path_buf, filter);
  if (found) {
    if (status) *status = YAI_SDK_CATALOG_RESOLVE_OK;
    return found;
  }

  found = yai_sdk_command_catalog_find_by_alias(cat, path_buf, &ambiguous_alias);
  if (found) {
    if (status) *status = YAI_SDK_CATALOG_RESOLVE_OK;
    return found;
  }

  if (ambiguous_alias) {
    if (status) *status = YAI_SDK_CATALOG_RESOLVE_AMBIGUOUS_ALIAS;
    return NULL;
  }

  if (!seen_entrypoint) {
    if (status) *status = YAI_SDK_CATALOG_RESOLVE_UNKNOWN_ENTRYPOINT;
    return NULL;
  }
  if (token_count > 1 && !seen_topic) {
    if (status) *status = YAI_SDK_CATALOG_RESOLVE_UNKNOWN_TOPIC;
    return NULL;
  }
  if (token_count > 2) {
    if (status) *status = YAI_SDK_CATALOG_RESOLVE_UNKNOWN_OP;
    return NULL;
  }

  if (status) *status = YAI_SDK_CATALOG_RESOLVE_NOT_FOUND;
  return NULL;
}

size_t yai_sdk_command_catalog_collect_entrypoints(
    const yai_sdk_command_catalog_t *cat,
    int surface_mask,
    const char **out_entrypoints,
    size_t out_cap)
{
  size_t n = 0;
  if (!cat || !out_entrypoints || out_cap == 0) return 0;
  if (surface_mask == 0) surface_mask = YAI_SDK_CATALOG_SURFACE_ALL;

  for (size_t i = 0; i < cat->command_count; i++) {
    const yai_sdk_command_ref_t *c = cat->commands_sorted[i];
    int seen = 0;
    if (!surface_matches(c->surface, surface_mask)) continue;
    for (size_t k = 0; k < n; k++) {
      if (strcmp(out_entrypoints[k], c->entrypoint) == 0) {
        seen = 1;
        break;
      }
    }
    if (seen) continue;
    if (n < out_cap) out_entrypoints[n] = c->entrypoint;
    n++;
  }

  if (n > 1 && n <= out_cap) {
    qsort((void *)out_entrypoints, n, sizeof(out_entrypoints[0]), cmp_entrypoint_cstr);
  }
  return n;
}

int yai_sdk_help_index_build(
    const yai_sdk_command_catalog_t *cat,
    const yai_sdk_catalog_filter_t *filter,
    yai_sdk_help_index_t *out)
{
  const yai_sdk_catalog_filter_t *use_filter = filter;
  yai_sdk_catalog_filter_t default_filter;
  const yai_sdk_command_ref_t **matches = NULL;
  size_t n = 0;

  if (!out) return 1;
  memset(out, 0, sizeof(*out));
  if (!cat) return 1;

  if (!use_filter) {
    memset(&default_filter, 0, sizeof(default_filter));
    default_filter.surface_mask = YAI_SDK_CATALOG_SURFACE_SURFACE;
    default_filter.stability_mask =
        YAI_SDK_CATALOG_STABILITY_STABLE |
        YAI_SDK_CATALOG_STABILITY_EXPERIMENTAL;
    default_filter.include_hidden = 0;
    default_filter.include_deprecated = 0;
    use_filter = &default_filter;
  }

  matches = (const yai_sdk_command_ref_t **)calloc(cat->command_count, sizeof(*matches));
  if (!matches) return 2;

  n = yai_sdk_command_catalog_query(cat, use_filter, matches, cat->command_count);
  if (n == 0) {
    free(matches);
    return 0;
  }

  for (size_t i = 0; i < n; i++) {
    const yai_sdk_command_ref_t *c = matches[i];
    yai_sdk_help_entrypoint_t *e = NULL;
    yai_sdk_help_topic_t *t = NULL;
    yai_sdk_help_op_t *slot;

    for (size_t ei = 0; ei < out->entrypoint_count; ei++) {
      if (strcmp(out->entrypoints[ei].entrypoint, c->entrypoint) == 0) {
        e = &out->entrypoints[ei];
        break;
      }
    }

    if (!e) {
      yai_sdk_help_entrypoint_t *ne = (yai_sdk_help_entrypoint_t *)realloc(
          out->entrypoints,
          (out->entrypoint_count + 1) * sizeof(*out->entrypoints));
      if (!ne) {
        free(matches);
        yai_sdk_help_index_free(out);
        return 3;
      }
      out->entrypoints = ne;
      e = &out->entrypoints[out->entrypoint_count++];
      memset(e, 0, sizeof(*e));
      snprintf(e->entrypoint, sizeof(e->entrypoint), "%s", c->entrypoint);
    }

    for (size_t ti = 0; ti < e->topic_count; ti++) {
      if (strcmp(e->topics[ti].topic, c->topic) == 0) {
        t = &e->topics[ti];
        break;
      }
    }

    if (!t) {
      yai_sdk_help_topic_t *nt = (yai_sdk_help_topic_t *)realloc(
          e->topics,
          (e->topic_count + 1) * sizeof(*e->topics));
      if (!nt) {
        free(matches);
        yai_sdk_help_index_free(out);
        return 4;
      }
      e->topics = nt;
      t = &e->topics[e->topic_count++];
      memset(t, 0, sizeof(*t));
      snprintf(t->topic, sizeof(t->topic), "%s", c->topic);
    }

    slot = (yai_sdk_help_op_t *)realloc(t->ops, (t->op_count + 1) * sizeof(*t->ops));
    if (!slot) {
      free(matches);
      yai_sdk_help_index_free(out);
      return 5;
    }
    t->ops = slot;
    t->ops[t->op_count].command = c;
    snprintf(t->ops[t->op_count].op,
             sizeof(t->ops[t->op_count].op),
             "%s",
             (c->op[0]) ? c->op : c->name);
    t->op_count++;
  }

  free(matches);
  return 0;
}

void yai_sdk_help_index_free(yai_sdk_help_index_t *idx)
{
  if (!idx) return;
  if (idx->entrypoints) {
    for (size_t i = 0; i < idx->entrypoint_count; i++) {
      yai_sdk_help_entrypoint_t *e = &idx->entrypoints[i];
      if (e->topics) {
        for (size_t j = 0; j < e->topic_count; j++) {
          free(e->topics[j].ops);
          e->topics[j].ops = NULL;
          e->topics[j].op_count = 0;
        }
      }
      free(e->topics);
      e->topics = NULL;
      e->topic_count = 0;
    }
  }
  free(idx->entrypoints);
  memset(idx, 0, sizeof(*idx));
}

const yai_sdk_help_entrypoint_t *yai_sdk_help_find_entrypoint(
    const yai_sdk_help_index_t *idx,
    const char *entrypoint)
{
  if (!idx || !entrypoint || !entrypoint[0]) return NULL;
  for (size_t i = 0; i < idx->entrypoint_count; i++) {
    if (strcmp(idx->entrypoints[i].entrypoint, entrypoint) == 0) return &idx->entrypoints[i];
  }
  return NULL;
}

const yai_sdk_help_topic_t *yai_sdk_help_find_topic(
    const yai_sdk_help_index_t *idx,
    const char *entrypoint,
    const char *topic)
{
  const yai_sdk_help_entrypoint_t *e;
  if (!idx || !entrypoint || !topic || !entrypoint[0] || !topic[0]) return NULL;
  e = yai_sdk_help_find_entrypoint(idx, entrypoint);
  if (!e) return NULL;
  for (size_t i = 0; i < e->topic_count; i++) {
    if (strcmp(e->topics[i].topic, topic) == 0) return &e->topics[i];
  }
  return NULL;
}

const yai_sdk_command_ref_t *yai_sdk_help_find_command(
    const yai_sdk_help_index_t *idx,
    const char *entrypoint,
    const char *topic,
    const char *op)
{
  const yai_sdk_help_topic_t *t = yai_sdk_help_find_topic(idx, entrypoint, topic);
  if (!t || !op || !op[0]) return NULL;
  for (size_t i = 0; i < t->op_count; i++) {
    if (strcmp(t->ops[i].op, op) == 0) return t->ops[i].command;
  }
  return NULL;
}

size_t yai_catalog_list_groups(
    const yai_catalog_t *cat,
    const yai_catalog_group_t **out_groups)
{
  if (out_groups) {
    *out_groups = (cat && cat->group_count > 0) ? cat->groups : NULL;
  }
  return (cat) ? cat->group_count : 0;
}

size_t yai_catalog_list_commands(
    const yai_catalog_t *cat,
    const char *group,
    const yai_catalog_command_t **out_commands)
{
  const yai_catalog_group_t *g;
  if (out_commands) *out_commands = NULL;
  g = yai_sdk_command_catalog_find_group(cat, group);
  if (!g) return 0;
  if (out_commands) *out_commands = g->commands;
  return g->command_count;
}

const yai_catalog_command_t *yai_catalog_find_by_id(
    const yai_catalog_t *cat,
    const char *canonical_id)
{
  return yai_sdk_command_catalog_find_by_id(cat, canonical_id);
}
