// SPDX-License-Identifier: Apache-2.0
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "yai/sdk/sdk.h"
#include "yai/sdk/registry/registry_registry.h"

int main(void)
{
  const char *law_root = getenv("YAI_LAW_ROOT");
  if (law_root && law_root[0] != '\0')
  {
    (void)setenv("YAI_SDK_COMPAT_REGISTRY_DIR", law_root, 1);
  }
  else
  {
    (void)setenv("YAI_SDK_COMPAT_REGISTRY_DIR", "../law", 1);
  }

  if (yai_sdk_abi_version() != YAI_SDK_ABI_VERSION)
  {
    fprintf(stderr, "sdk_smoke: ABI mismatch runtime=%d compile=%d\n",
            yai_sdk_abi_version(), YAI_SDK_ABI_VERSION);
    return 1;
  }
  if (strcmp(yai_sdk_errstr(YAI_SDK_SERVER_OFF), "server_off") != 0)
  {
    fprintf(stderr, "sdk_smoke: errstr mapping failed for YAI_SDK_SERVER_OFF\n");
    return 1;
  }

  {
    char ws[64] = {0};
    if (yai_sdk_container_context_clear_binding() != 0)
    {
      fprintf(stderr, "sdk_smoke: context clear failed\n");
      return 1;
    }
    if (yai_sdk_container_context_current(ws, sizeof(ws)) != 1)
    {
      fprintf(stderr, "sdk_smoke: context should be empty after clear\n");
      return 1;
    }
    if (yai_sdk_container_context_bind("ws_smoke_ctx") != 0)
    {
      fprintf(stderr, "sdk_smoke: context set failed\n");
      return 1;
    }
    if (yai_sdk_container_context_current(ws, sizeof(ws)) != 0 || strcmp(ws, "ws_smoke_ctx") != 0)
    {
      fprintf(stderr, "sdk_smoke: context get mismatch\n");
      return 1;
    }
    memset(ws, 0, sizeof(ws));
    if (yai_sdk_container_context_resolve(NULL, ws, sizeof(ws)) != 0 || strcmp(ws, "ws_smoke_ctx") != 0)
    {
      fprintf(stderr, "sdk_smoke: context resolve mismatch\n");
      return 1;
    }
    if (yai_sdk_container_context_clear_binding() != 0)
    {
      fprintf(stderr, "sdk_smoke: context clear failed (post)\n");
      return 1;
    }
  }

  /* 1) Offline registry lookup must work */
  if (yai_law_registry_init() != 0)
  {
    fprintf(stderr, "sdk_smoke: registry init failed\n");
    return 2;
  }
  const yai_law_registry_t *reg = yai_law_registry();
  const yai_law_command_t *c = NULL;
  yai_sdk_command_catalog_t catalog = {0};
  const yai_sdk_command_ref_t *catalog_cmd = NULL;
  if (!reg || reg->commands_len == 0)
  {
    fprintf(stderr, "sdk_smoke: registry is empty\n");
    return 2;
  }
  c = &reg->commands[0];
  if (yai_sdk_command_catalog_load(&catalog) != 0)
  {
    fprintf(stderr, "sdk_smoke: catalog load failed\n");
    return 2;
  }
  catalog_cmd = yai_sdk_command_catalog_find_by_id(&catalog, c->id);
  if (!catalog_cmd)
  {
    fprintf(stderr, "sdk_smoke: catalog lookup failed for %s\n", c->id);
    yai_sdk_command_catalog_free(&catalog);
    return 2;
  }

  /* 2) Deterministic server-off semantics (always) */
  (void)setenv("YAI_RUNTIME_SOCK", "/tmp/yai-runtime-sock-does-not-exist.sock", 1);

  yai_sdk_client_t *client = NULL;
  yai_sdk_client_opts_t opts = {
      .ws_id = "default",
      .uds_path = NULL,
      .arming = 1,
      .role = "operator",
      .auto_handshake = 1,
  };
  int rc = yai_sdk_client_open(&client, &opts);

  if (rc != YAI_SDK_SERVER_OFF)
  {
    fprintf(stderr, "sdk_smoke: expected YAI_SDK_SERVER_OFF=%d, got rc=%d\n", YAI_SDK_SERVER_OFF, rc);
    return 3;
  }

  /* 3) Optional online probe (does NOT fail CI by default) */
  const char *online = getenv("YAI_SDK_SMOKE_ONLINE");
  if (online && strcmp(online, "1") == 0)
  {
    (void)unsetenv("YAI_RUNTIME_SOCK"); /* use default path */
    rc = yai_sdk_client_open(&client, &opts);
    if (rc != 0)
    {
      fprintf(stderr, "sdk_smoke: online ping failed rc=%d\n", rc);
      yai_sdk_command_catalog_free(&catalog);
      return 4;
    }

    yai_sdk_reply_t out = {0};
    yai_sdk_control_call_t call = {
        .target_plane = YAI_SDK_RUNTIME_TARGET_PLANE,
        .command_id = c->id,
        .argv = NULL,
        .argv_len = 0,
    };
    rc = yai_sdk_client_call(client, &call, &out);
    yai_sdk_client_close(client);
    if (rc != 0)
    {
      fprintf(stderr, "sdk_smoke: online call failed rc=%d code=%s reason=%s\n", rc, out.code, out.reason);
      yai_sdk_reply_free(&out);
      yai_sdk_command_catalog_free(&catalog);
      return 5;
    }
    if (!out.exec_reply_json || !out.exec_reply_json[0])
    {
      fprintf(stderr, "sdk_smoke: expected exec reply json\n");
      yai_sdk_reply_free(&out);
      yai_sdk_command_catalog_free(&catalog);
      return 6;
    }
    if (!out.summary[0])
    {
      fprintf(stderr, "sdk_smoke: expected reply summary\n");
      yai_sdk_reply_free(&out);
      yai_sdk_command_catalog_free(&catalog);
      return 7;
    }
    yai_sdk_reply_free(&out);
  }

  yai_sdk_command_catalog_free(&catalog);
  printf("sdk_smoke: ok (abi=%d version=%s registry=%s, server_off_rc=%d)\n",
         yai_sdk_abi_version(), yai_sdk_version(), c->id, YAI_SDK_SERVER_OFF);
  return 0;
}
