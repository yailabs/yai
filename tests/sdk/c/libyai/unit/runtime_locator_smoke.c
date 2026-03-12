/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/sdk/client.h>
#include <yai/sdk/errors.h>
#include <yai/sdk/transport.h>
#include <yai/sdk/paths.h>

#include <stdio.h>
#include <string.h>

int main(void)
{
  char buf[1024];
  char err[128];
  yai_runtime_deploy_mode_t mode = YAI_RUNTIME_DEPLOY_UNKNOWN;
  yai_sdk_runtime_endpoint_t endpoint;
  yai_sdk_runtime_locator_state_t state;
  yai_sdk_client_t *client = NULL;
  yai_sdk_client_opts_t opts = {0};

  if (yai_path_runtime_home(buf, sizeof(buf)) != 0 || !buf[0]) {
    fprintf(stderr, "runtime_home resolution failed\n");
    return 1;
  }

  if (yai_path_detect_deploy_mode(&mode) != 0) {
    fprintf(stderr, "deploy_mode resolution failed\n");
    return 1;
  }

  if (yai_path_runtime_ingress_sock(buf, sizeof(buf)) != 0 || !buf[0]) {
    fprintf(stderr, "runtime ingress socket resolution failed\n");
    return 1;
  }

  /* Runtime binary may be unresolved on minimal test hosts; ensure deterministic API behavior. */
  (void)yai_path_runtime_bin(buf, sizeof(buf));

  if (yai_sdk_runtime_endpoint_local_default(&endpoint) != 0) {
    fprintf(stderr, "local default endpoint init failed\n");
    return 1;
  }
  err[0] = '\0';
  if (yai_sdk_runtime_endpoint_resolve(&endpoint, &state, err, sizeof(err)) != 0) {
    fprintf(stderr, "local default resolve failed: %s\n", err);
    return 1;
  }
  if (state.endpoint_kind != YAI_SDK_ENDPOINT_KIND_LOCAL_DEFAULT ||
      state.transport_kind != YAI_SDK_TRANSPORT_KIND_UDS ||
      state.explicit_target != 0 ||
      !state.resolved_ingress[0]) {
    fprintf(stderr, "local default resolve state mismatch\n");
    return 1;
  }

  if (yai_sdk_runtime_endpoint_local_uds("/tmp/yai-sdk-runtime.sock", &endpoint) != 0) {
    fprintf(stderr, "local uds endpoint init failed\n");
    return 1;
  }
  err[0] = '\0';
  if (yai_sdk_runtime_endpoint_resolve(&endpoint, &state, err, sizeof(err)) != 0) {
    fprintf(stderr, "local uds resolve failed: %s\n", err);
    return 1;
  }
  if (state.endpoint_kind != YAI_SDK_ENDPOINT_KIND_LOCAL_UDS ||
      state.transport_kind != YAI_SDK_TRANSPORT_KIND_UDS ||
      state.explicit_target != 1 ||
      strcmp(state.resolved_ingress, "/tmp/yai-sdk-runtime.sock") != 0) {
    fprintf(stderr, "local uds resolve state mismatch\n");
    return 1;
  }

  if (yai_sdk_runtime_endpoint_owner_ref("uds:///tmp/yai-owner.sock", &endpoint) != 0) {
    fprintf(stderr, "owner ref endpoint init failed\n");
    return 1;
  }
  err[0] = '\0';
  if (yai_sdk_runtime_endpoint_resolve(&endpoint, &state, err, sizeof(err)) != 0) {
    fprintf(stderr, "owner ref resolve failed: %s\n", err);
    return 1;
  }
  if (state.endpoint_kind != YAI_SDK_ENDPOINT_KIND_OWNER_REF ||
      state.transport_kind != YAI_SDK_TRANSPORT_KIND_UDS ||
      state.explicit_target != 1 ||
      strcmp(state.resolved_ingress, "/tmp/yai-owner.sock") != 0) {
    fprintf(stderr, "owner ref resolve state mismatch\n");
    return 1;
  }

  if (yai_sdk_runtime_endpoint_owner_ref("tcp://127.0.0.1:9443", &endpoint) != 0) {
    fprintf(stderr, "owner ref endpoint init for unsupported transport failed\n");
    return 1;
  }
  err[0] = '\0';
  if (yai_sdk_runtime_endpoint_resolve(&endpoint, &state, err, sizeof(err)) != -3 ||
      strcmp(err, "runtime_transport_unsupported") != 0) {
    fprintf(stderr, "unsupported transport classification mismatch\n");
    return 1;
  }

  /* Client-open error classification should surface endpoint model errors clearly. */
  opts.ws_id = "locator-smoke";
  opts.owner_endpoint_ref = "tcp://127.0.0.1:9443";
  if (yai_sdk_client_open(&client, &opts) != YAI_SDK_TRANSPORT_UNSUPPORTED) {
    fprintf(stderr, "client open transport unsupported classification mismatch\n");
    return 1;
  }

  memset(&opts, 0, sizeof(opts));
  opts.ws_id = "locator-smoke";
  opts.runtime_endpoint = &endpoint;
  if (yai_sdk_runtime_endpoint_owner_ref("uds:///tmp/yai-owner-smoke.sock", &endpoint) != 0) {
    fprintf(stderr, "owner endpoint build failed\n");
    return 1;
  }
  if (yai_sdk_client_open(&client, &opts) != YAI_SDK_SERVER_OFF) {
    fprintf(stderr, "explicit owner target should reach connect path and return server_off in smoke env\n");
    return 1;
  }

  return 0;
}
