// SPDX-License-Identifier: Apache-2.0
#include <stdio.h>
#include <string.h>

#include "yai/sdk/sdk.h"

int main(void)
{
  yai_sdk_reply_t out = {0};
  int rc = 0;

  if (!yai_sdk_data_is_query_family("events") ||
      !yai_sdk_data_is_query_family("evidence") ||
      !yai_sdk_cognition_is_query_family("transient") ||
      !yai_sdk_cognition_is_query_family("providers") ||
      !yai_sdk_graph_is_query_family("graph")) {
    fprintf(stderr, "workspace_typed_surface_smoke: query family guards mismatch\n");
    return 1;
  }

  rc = yai_sdk_container_context_status(NULL, &out);
  if (rc != YAI_SDK_BAD_ARGS) return 2;
  rc = yai_sdk_container_graph_recent(NULL, &out);
  if (rc != YAI_SDK_BAD_ARGS) return 3;
  rc = yai_sdk_container_db_count(NULL, &out);
  if (rc != YAI_SDK_BAD_ARGS) return 4;
  rc = yai_sdk_container_data_governance(NULL, &out);
  if (rc != YAI_SDK_BAD_ARGS) return 5;
  rc = yai_sdk_container_cognition_memory(NULL, &out);
  if (rc != YAI_SDK_BAD_ARGS) return 6;
  rc = yai_sdk_container_policy_effective(NULL, &out);
  if (rc != YAI_SDK_BAD_ARGS) return 7;
  rc = yai_sdk_container_domain_get(NULL, &out);
  if (rc != YAI_SDK_BAD_ARGS) return 8;
  rc = yai_sdk_container_recovery_status(NULL, &out);
  if (rc != YAI_SDK_BAD_ARGS) return 9;
  rc = yai_sdk_container_debug_resolution(NULL, &out);
  if (rc != YAI_SDK_BAD_ARGS) return 10;

  rc = yai_sdk_container_domain_set(NULL, NULL, NULL, &out);
  if (rc != YAI_SDK_BAD_ARGS) return 11;
  rc = yai_sdk_container_policy_attach(NULL, NULL, &out);
  if (rc != YAI_SDK_BAD_ARGS) return 12;
  rc = yai_sdk_container_recovery_reopen(NULL, "", &out);
  if (rc != YAI_SDK_BAD_ARGS) return 13;

  if (strcmp(YAI_SDK_DEBUG_CMD_RESOLUTION, "yai.container.debug_resolution") != 0) return 14;
  if (strcmp(YAI_SDK_RECOVERY_CMD_REOPEN, "yai.container.open") != 0) return 15;

  puts("workspace_typed_surface_smoke: ok");
  return 0;
}
