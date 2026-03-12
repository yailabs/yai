// SPDX-License-Identifier: Apache-2.0
#include <stdio.h>

#include "yai/sdk/sdk.h"

int main(void)
{
  yai_sdk_client_t *client = NULL;
  yai_sdk_client_opts_t opts = {
      .ws_id = "source-demo",
      .auto_handshake = 1,
      .arming = 1,
      .role = "operator",
  };
  yai_sdk_source_enroll_request_t req = {
      .container_id = "source-demo",
      .source_label = "edge-node-a",
      .owner_ref = "uds:///tmp/yai-owner.sock",
  };
  yai_sdk_reply_t reply = {0};
  yai_sdk_source_enroll_reply_t parsed;
  int rc = yai_sdk_client_open(&client, &opts);
  if (rc != YAI_SDK_OK) {
    fprintf(stderr, "open failed: %s\n", yai_sdk_errstr((yai_sdk_err_t)rc));
    return rc;
  }

  rc = yai_sdk_source_enroll(client, &req, &reply);
  if (rc != YAI_SDK_OK) {
    fprintf(stderr, "source enroll failed: code=%s reason=%s\n", reply.code, reply.reason);
    yai_sdk_reply_free(&reply);
    yai_sdk_client_close(client);
    return rc;
  }

  if (yai_sdk_source_enroll_reply_from_sdk(&reply, &parsed) == YAI_SDK_OK) {
    printf("source enrolled: ws=%s node=%s daemon=%s route=%s\n",
           parsed.container_id,
           parsed.source_node_id,
           parsed.daemon_instance_id,
           parsed.mediation.route);
  }

  yai_sdk_reply_free(&reply);
  yai_sdk_client_close(client);
  return 0;
}
