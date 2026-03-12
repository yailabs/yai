/* SPDX-License-Identifier: Apache-2.0 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "yai/sdk/sdk.h"

int main(void)
{
  yai_sdk_command_catalog_t cat = {0};
  const yai_sdk_command_ref_t *c = NULL;
  const yai_sdk_command_ref_t *results[16];
  yai_sdk_catalog_filter_t filter = {0};
  const char *path_tokens[3] = {0};
  yai_sdk_catalog_resolve_status_t rstatus = YAI_SDK_CATALOG_RESOLVE_BAD_ARGS;
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
    fprintf(stderr, "catalog_smoke: load failed rc=%d\n", rc);
    return 1;
  }
  if (cat.group_count == 0) {
    fprintf(stderr, "catalog_smoke: empty catalog\n");
    yai_sdk_command_catalog_free(&cat);
    return 2;
  }
  if (cat.command_count == 0) {
    fprintf(stderr, "catalog_smoke: expected command_count > 0\n");
    yai_sdk_command_catalog_free(&cat);
    return 2;
  }

  c = cat.command_count > 0 ? cat.commands_sorted[0] : NULL;
  if (!c || !c->id[0]) {
    fprintf(stderr, "catalog_smoke: expected at least one command id\n");
    yai_sdk_command_catalog_free(&cat);
    return 3;
  }
  c = yai_sdk_command_catalog_find_by_id(&cat, c->id);
  if (!c) {
    fprintf(stderr, "catalog_smoke: expected command lookup by id\n");
    yai_sdk_command_catalog_free(&cat);
    return 3;
  }
  if (!c->entrypoint[0] || !c->topic[0] || !c->op[0]) {
    fprintf(stderr, "catalog_smoke: taxonomy fields should be non-empty\n");
    yai_sdk_command_catalog_free(&cat);
    return 4;
  }

  c = yai_sdk_command_catalog_find_by_canonical_path(&cat, c->canonical_path, NULL);
  if (!c) {
    fprintf(stderr, "catalog_smoke: expected canonical path lookup\n");
    yai_sdk_command_catalog_free(&cat);
    return 5;
  }
  path_tokens[0] = c->entrypoint;
  path_tokens[1] = c->topic;
  path_tokens[2] = c->op;
  c = yai_sdk_command_catalog_resolve_path(&cat, path_tokens, 3, NULL, &rstatus);
  if (!c || rstatus != YAI_SDK_CATALOG_RESOLVE_OK) {
    fprintf(stderr, "catalog_smoke: resolve_path failed status=%d\n", (int)rstatus);
    yai_sdk_command_catalog_free(&cat);
    return 6;
  }

  filter.surface_mask = YAI_SDK_CATALOG_SURFACE_SURFACE;
  filter.stability_mask = YAI_SDK_CATALOG_STABILITY_STABLE | YAI_SDK_CATALOG_STABILITY_EXPERIMENTAL;
  filter.include_hidden = 0;
  filter.include_deprecated = 0;
  if (yai_sdk_command_catalog_query(&cat, &filter, results, 16) == 0) {
    /* Registry waves may classify surface commands as planned first. */
    filter.surface_mask = YAI_SDK_CATALOG_SURFACE_ALL;
    filter.stability_mask = YAI_SDK_CATALOG_STABILITY_ALL;
    if (yai_sdk_command_catalog_query(&cat, &filter, results, 16) == 0) {
      fprintf(stderr, "catalog_smoke: expected non-empty filtered query\n");
      yai_sdk_command_catalog_free(&cat);
      return 7;
    }
  }

  yai_sdk_command_catalog_free(&cat);
  puts("catalog_smoke: ok");
  return 0;
}
