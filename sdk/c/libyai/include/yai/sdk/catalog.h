/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum yai_sdk_catalog_surface_mask {
  YAI_SDK_CATALOG_SURFACE_SURFACE = 1 << 0,
  YAI_SDK_CATALOG_SURFACE_ANCILLARY = 1 << 1,
  YAI_SDK_CATALOG_SURFACE_PLUMBING = 1 << 2,
  YAI_SDK_CATALOG_SURFACE_ALL =
      YAI_SDK_CATALOG_SURFACE_SURFACE |
      YAI_SDK_CATALOG_SURFACE_ANCILLARY |
      YAI_SDK_CATALOG_SURFACE_PLUMBING,
} yai_sdk_catalog_surface_mask_t;

typedef enum yai_sdk_catalog_stability_mask {
  YAI_SDK_CATALOG_STABILITY_STABLE = 1 << 0,
  YAI_SDK_CATALOG_STABILITY_EXPERIMENTAL = 1 << 1,
  YAI_SDK_CATALOG_STABILITY_PLANNED = 1 << 2,
  YAI_SDK_CATALOG_STABILITY_DEPRECATED = 1 << 3,
  YAI_SDK_CATALOG_STABILITY_ALL =
      YAI_SDK_CATALOG_STABILITY_STABLE |
      YAI_SDK_CATALOG_STABILITY_EXPERIMENTAL |
      YAI_SDK_CATALOG_STABILITY_PLANNED |
      YAI_SDK_CATALOG_STABILITY_DEPRECATED,
} yai_sdk_catalog_stability_mask_t;

typedef struct yai_sdk_command_ref {
  char group[32];
  char name[64];
  char id[128];
  char summary[160];

  char surface[16];
  char entrypoint[32];
  char topic[64];
  char op[64];
  char domain[32];
  char layer[16];
  char stability[16];
  char canonical_path[160];
  int help_order;
  int hidden;
  int deprecated;
  char replaced_by[128];
  const char **aliases;
  size_t aliases_len;
  const char **outputs;
  size_t outputs_len;
  const char **side_effects;
  size_t side_effects_len;
} yai_sdk_command_ref_t;

typedef struct yai_sdk_command_group {
  char group[32];
  yai_sdk_command_ref_t *commands;
  size_t command_count;
} yai_sdk_command_group_t;

typedef struct yai_sdk_command_catalog {
  yai_sdk_command_group_t *groups;
  size_t group_count;
  yai_sdk_command_ref_t **commands_sorted;
  size_t command_count;
} yai_sdk_command_catalog_t;

typedef yai_sdk_command_catalog_t yai_catalog_t;
typedef yai_sdk_command_group_t yai_catalog_group_t;
typedef yai_sdk_command_ref_t yai_catalog_command_t;

typedef struct yai_sdk_catalog_filter {
  int surface_mask;   /* yai_sdk_catalog_surface_mask_t */
  int stability_mask; /* yai_sdk_catalog_stability_mask_t */
  const char *entrypoint;
  const char *topic;
  int include_aliases;
  int include_hidden;
  int include_deprecated;
} yai_sdk_catalog_filter_t;

typedef enum yai_sdk_catalog_resolve_status {
  YAI_SDK_CATALOG_RESOLVE_OK = 0,
  YAI_SDK_CATALOG_RESOLVE_BAD_ARGS = 1,
  YAI_SDK_CATALOG_RESOLVE_UNKNOWN_ENTRYPOINT = 2,
  YAI_SDK_CATALOG_RESOLVE_UNKNOWN_TOPIC = 3,
  YAI_SDK_CATALOG_RESOLVE_UNKNOWN_OP = 4,
  YAI_SDK_CATALOG_RESOLVE_AMBIGUOUS_ALIAS = 5,
  YAI_SDK_CATALOG_RESOLVE_NOT_FOUND = 6,
} yai_sdk_catalog_resolve_status_t;

typedef struct yai_sdk_help_op {
  char op[64];
  const yai_sdk_command_ref_t *command;
} yai_sdk_help_op_t;

typedef struct yai_sdk_help_topic {
  char topic[64];
  yai_sdk_help_op_t *ops;
  size_t op_count;
} yai_sdk_help_topic_t;

typedef struct yai_sdk_help_entrypoint {
  char entrypoint[32];
  yai_sdk_help_topic_t *topics;
  size_t topic_count;
} yai_sdk_help_entrypoint_t;

typedef struct yai_sdk_help_index {
  yai_sdk_help_entrypoint_t *entrypoints;
  size_t entrypoint_count;
} yai_sdk_help_index_t;

int yai_sdk_command_catalog_load(yai_sdk_command_catalog_t *out);
void yai_sdk_command_catalog_free(yai_sdk_command_catalog_t *cat);

const yai_sdk_command_group_t *yai_sdk_command_catalog_find_group(
    const yai_sdk_command_catalog_t *cat,
    const char *group);

const yai_sdk_command_ref_t *yai_sdk_command_catalog_find_command(
    const yai_sdk_command_catalog_t *cat,
    const char *group,
    const char *name);

const yai_sdk_command_ref_t *yai_sdk_command_catalog_find_by_id(
    const yai_sdk_command_catalog_t *cat,
    const char *canonical_id);

const yai_sdk_command_ref_t *yai_sdk_command_catalog_find_by_path(
    const yai_sdk_command_catalog_t *cat,
    const char *entrypoint,
    const char *topic,
    const char *op,
    int surface_mask);

const yai_sdk_command_ref_t *yai_sdk_command_catalog_find_by_canonical_path(
    const yai_sdk_command_catalog_t *cat,
    const char *canonical_path,
    const yai_sdk_catalog_filter_t *filter);

const yai_sdk_command_ref_t *yai_sdk_command_catalog_find_by_alias(
    const yai_sdk_command_catalog_t *cat,
    const char *alias,
    int *ambiguous);

const yai_sdk_command_ref_t *yai_sdk_command_catalog_resolve_alias(
    const yai_sdk_command_catalog_t *cat,
    const char *alias,
    yai_sdk_catalog_resolve_status_t *status);

size_t yai_sdk_command_catalog_query(
    const yai_sdk_command_catalog_t *cat,
    const yai_sdk_catalog_filter_t *filter,
    const yai_sdk_command_ref_t **out_matches,
    size_t out_cap);

const yai_sdk_command_ref_t *yai_sdk_command_catalog_resolve_path(
    const yai_sdk_command_catalog_t *cat,
    const char **tokens,
    size_t token_count,
    const yai_sdk_catalog_filter_t *filter,
    yai_sdk_catalog_resolve_status_t *status);

size_t yai_sdk_command_catalog_collect_entrypoints(
    const yai_sdk_command_catalog_t *cat,
    int surface_mask,
    const char **out_entrypoints,
    size_t out_cap);

int yai_sdk_help_index_build(
    const yai_sdk_command_catalog_t *cat,
    const yai_sdk_catalog_filter_t *filter,
    yai_sdk_help_index_t *out);

void yai_sdk_help_index_free(yai_sdk_help_index_t *idx);

const yai_sdk_help_entrypoint_t *yai_sdk_help_find_entrypoint(
    const yai_sdk_help_index_t *idx,
    const char *entrypoint);

const yai_sdk_help_topic_t *yai_sdk_help_find_topic(
    const yai_sdk_help_index_t *idx,
    const char *entrypoint,
    const char *topic);

const yai_sdk_command_ref_t *yai_sdk_help_find_command(
    const yai_sdk_help_index_t *idx,
    const char *entrypoint,
    const char *topic,
    const char *op);

/* Canonical short aliases (stable surface for CLI/UI layers). */
size_t yai_catalog_list_groups(
    const yai_catalog_t *cat,
    const yai_catalog_group_t **out_groups);

size_t yai_catalog_list_commands(
    const yai_catalog_t *cat,
    const char *group,
    const yai_catalog_command_t **out_commands);

const yai_catalog_command_t *yai_catalog_find_by_id(
    const yai_catalog_t *cat,
    const char *canonical_id);

#ifdef __cplusplus
}
#endif
