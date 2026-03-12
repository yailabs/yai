/* SPDX-License-Identifier: Apache-2.0 */
#include <stdio.h>

#include <yai/sdk/sdk.h>

int main(void)
{
    yai_sdk_client_t *client = NULL;
    yai_sdk_client_opts_t opts = {0};
    yai_sdk_reply_t reply = {0};
    yai_sdk_control_call_t call;
    const char *argvv[] = {"--family", YAI_SDK_DATA_QUERY_ENFORCEMENT};
    yai_sdk_runtime_state_t runtime_state;
    yai_sdk_governance_state_t governance_state;
    int rc = 0;

    opts.ws_id = "default";
    opts.role = "operator";
    opts.arming = 1;
    opts.auto_handshake = 1;
    opts.correlation_id = "example-custom";

    call.target_plane = YAI_SDK_RUNTIME_TARGET_PLANE;
    call.command_id = YAI_SDK_DATA_CMD_QUERY;
    call.argv = argvv;
    call.argv_len = 2;

    rc = yai_sdk_client_open(&client, &opts);
    if (rc != YAI_SDK_OK) {
        fprintf(stderr, "open failed: %s\n", yai_sdk_errstr((yai_sdk_err_t)rc));
        return rc;
    }

    rc = yai_sdk_client_call(client, &call, &reply);
    if (rc != YAI_SDK_OK) {
        fprintf(stderr, "call failed: %s\n", yai_sdk_errstr((yai_sdk_err_t)rc));
    } else {
        printf("command=%s code=%s summary=%s\n", reply.command_id, reply.code, reply.summary);
        if (yai_sdk_reply_runtime_state(&reply, &runtime_state) == YAI_SDK_OK) {
            printf("container=%s binding=%d data_ready=%d\n",
                   runtime_state.container_id,
                   (int)runtime_state.container_binding,
                   runtime_state.data.ready);
        }
        if (yai_sdk_reply_governance_state(&reply, &governance_state) == YAI_SDK_OK) {
            printf("governance effect=%s review=%s blocked=%d\n",
                   governance_state.effect,
                   governance_state.review_state,
                   governance_state.blocked);
        }
        if (reply.exec_reply_json) {
            printf("exec_reply=%s\n", reply.exec_reply_json);
        }
    }

    yai_sdk_reply_free(&reply);
    yai_sdk_client_close(client);
    return rc;
}
