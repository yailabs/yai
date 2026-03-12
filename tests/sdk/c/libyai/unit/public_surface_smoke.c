// SPDX-License-Identifier: Apache-2.0
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#include "yai/sdk/sdk.h"

int main(void)
{
  const char *law_root = getenv("YAI_LAW_ROOT");
  yai_sdk_command_catalog_t catalog = {0};
  yai_sdk_catalog_filter_t filter = {
      .surface_mask = YAI_SDK_CATALOG_SURFACE_SURFACE,
      .stability_mask = YAI_SDK_CATALOG_STABILITY_STABLE,
  };
  const yai_sdk_command_ref_t *matches[1] = {0};

  if (yai_sdk_abi_version() != YAI_SDK_ABI_VERSION) {
    fprintf(stderr, "public_surface_smoke: abi mismatch\n");
    return 1;
  }
  if (!yai_sdk_version() || !yai_sdk_version()[0]) {
    fprintf(stderr, "public_surface_smoke: empty version\n");
    return 1;
  }

  /* Canonical SDK-1 unified-runtime taxonomy checks. */
  if (strcmp(YAI_SDK_FAMILY_EXEC, "exec") != 0 ||
      strcmp(YAI_SDK_FAMILY_DATA, "data") != 0 ||
      strcmp(YAI_SDK_FAMILY_GRAPH, "graph") != 0 ||
      strcmp(YAI_SDK_FAMILY_COGNITION, "cognition") != 0) {
    fprintf(stderr, "public_surface_smoke: family constants mismatch\n");
    return 1;
  }
  if (strcmp(YAI_SDK_CMD_RUNTIME_PING, "yai.runtime.ping") != 0 ||
      strcmp(YAI_SDK_EXEC_CMD_RUN_ACTION, "yai.container.run") != 0) {
    fprintf(stderr, "public_surface_smoke: command constants mismatch\n");
    return 1;
  }
  if (strcmp(YAI_SDK_GRAPH_CMD_LINEAGE, "yai.container.graph.lineage") != 0 ||
      strcmp(YAI_SDK_DB_CMD_TAIL, "yai.container.events.tail") != 0 ||
      strcmp(YAI_SDK_POLICY_CMD_EFFECTIVE, "yai.container.policy_effective") != 0 ||
      strcmp(YAI_SDK_RECOVERY_CMD_LOAD, "yai.container.lifecycle.maintain") != 0) {
    fprintf(stderr, "public_surface_smoke: container family constants mismatch\n");
    return 1;
  }

  {
    yai_sdk_reply_t out = {0};
    int rc_null = yai_sdk_container_graph_summary(NULL, &out);
    if (rc_null != YAI_SDK_BAD_ARGS) {
      fprintf(stderr, "public_surface_smoke: expected BAD_ARGS for null client in typed helper, got %d\n", rc_null);
      return 1;
    }
    rc_null = yai_sdk_container_policy_attach(NULL, "customer.default.org-container-contextual-review", &out);
    if (rc_null != YAI_SDK_BAD_ARGS) {
      fprintf(stderr, "public_surface_smoke: expected BAD_ARGS for null client in policy helper, got %d\n", rc_null);
      return 1;
    }
    rc_null = yai_sdk_container_domain_set(NULL, "scientific", "parameter-governance", &out);
    if (rc_null != YAI_SDK_BAD_ARGS) {
      fprintf(stderr, "public_surface_smoke: expected BAD_ARGS for null client in domain helper, got %d\n", rc_null);
      return 1;
    }
  }

  if (law_root && law_root[0] != '\0') {
    (void)setenv("YAI_SDK_COMPAT_REGISTRY_DIR", law_root, 1);
  } else {
    (void)setenv("YAI_SDK_COMPAT_REGISTRY_DIR", "../law", 1);
  }

  if (yai_sdk_command_catalog_load(&catalog) != 0) {
    fprintf(stderr, "public_surface_smoke: catalog load failed\n");
    return 2;
  }

  if (yai_sdk_command_catalog_query(&catalog, &filter, matches, 1) == 0) {
    fprintf(stderr, "public_surface_smoke: expected at least one stable surface command\n");
    yai_sdk_command_catalog_free(&catalog);
    return 2;
  }

  yai_sdk_command_catalog_free(&catalog);

  if (strcmp(yai_sdk_errstr(YAI_SDK_SERVER_OFF), "server_off") != 0) {
    fprintf(stderr, "public_surface_smoke: errstr mismatch\n");
    return 3;
  }

  printf("public_surface_smoke: ok\n");
  return 0;
}
