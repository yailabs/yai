/* SPDX-License-Identifier: Apache-2.0 */

#include <stdio.h>
#include <stdlib.h>

#include "yai/sdk/sdk.h"

int main(void)
{
  yai_sdk_command_catalog_t cat = {0};
  yai_sdk_help_index_t idx = {0};
  yai_sdk_catalog_filter_t filter = {0};
  const yai_sdk_help_entrypoint_t *entry = NULL;
  const yai_sdk_help_topic_t *topic = NULL;
  const yai_sdk_command_ref_t *cmd = NULL;
  int rc;

  {
    const char *law_root = getenv("YAI_LAW_ROOT");
    if (law_root && law_root[0] != '\0') {
      (void)setenv("YAI_SDK_COMPAT_REGISTRY_DIR", law_root, 1);
    } else {
      (void)setenv("YAI_SDK_COMPAT_REGISTRY_DIR", "../law", 1);
    }
  }

  rc = yai_sdk_command_catalog_load(&cat);
  if (rc != 0) {
    fprintf(stderr, "help_index_smoke: catalog load failed rc=%d\n", rc);
    return 1;
  }

  filter.surface_mask = YAI_SDK_CATALOG_SURFACE_SURFACE;
  filter.stability_mask = YAI_SDK_CATALOG_STABILITY_STABLE | YAI_SDK_CATALOG_STABILITY_EXPERIMENTAL;
  filter.include_hidden = 0;
  filter.include_deprecated = 0;

  rc = yai_sdk_help_index_build(&cat, &filter, &idx);
  if (rc != 0) {
    fprintf(stderr, "help_index_smoke: build failed rc=%d\n", rc);
    yai_sdk_command_catalog_free(&cat);
    return 2;
  }
  if (idx.entrypoint_count == 0) {
    yai_sdk_help_index_free(&idx);
    filter.surface_mask = YAI_SDK_CATALOG_SURFACE_ALL;
    filter.stability_mask = YAI_SDK_CATALOG_STABILITY_ALL;
    rc = yai_sdk_help_index_build(&cat, &filter, &idx);
    if (rc != 0 || idx.entrypoint_count == 0) {
      fprintf(stderr, "help_index_smoke: empty index\n");
      yai_sdk_help_index_free(&idx);
      yai_sdk_command_catalog_free(&cat);
      return 3;
    }
  }

  entry = &idx.entrypoints[0];
  if (!entry || !entry->entrypoint[0]) {
    fprintf(stderr, "help_index_smoke: expected at least one entrypoint\n");
    yai_sdk_help_index_free(&idx);
    yai_sdk_command_catalog_free(&cat);
    return 4;
  }

  topic = entry->topic_count > 0 ? &entry->topics[0] : NULL;
  if (!topic || !topic->topic[0]) {
    fprintf(stderr, "help_index_smoke: expected at least one topic\n");
    yai_sdk_help_index_free(&idx);
    yai_sdk_command_catalog_free(&cat);
    return 5;
  }

  cmd = topic->op_count > 0 ? topic->ops[0].command : NULL;
  if (!cmd || !cmd->id[0]) {
    fprintf(stderr, "help_index_smoke: expected at least one command\n");
    yai_sdk_help_index_free(&idx);
    yai_sdk_command_catalog_free(&cat);
    return 6;
  }

  yai_sdk_help_index_free(&idx);
  yai_sdk_command_catalog_free(&cat);
  puts("help_index_smoke: ok");
  return 0;
}
