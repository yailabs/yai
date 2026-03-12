/* SPDX-License-Identifier: Apache-2.0 */
#include <stdio.h>

#include <yai/sdk/sdk.h>

static void print_step(const char *label, int rc, const yai_sdk_reply_t *reply)
{
    printf("%-22s rc=%d", label, rc);
    if (reply && reply->code[0]) {
        printf(" code=%s reason=%s", reply->code, reply->reason);
    }
    putchar('\n');
}

int main(void)
{
    yai_sdk_client_t *client = NULL;
    yai_sdk_client_opts_t opts = {
        .ws_id = "ws_vertical_demo",
        .role = "operator",
        .arming = 1,
        .auto_handshake = 1,
    };
    yai_sdk_reply_t reply = {0};
    int rc = yai_sdk_client_open(&client, &opts);

    if (rc != YAI_SDK_OK) {
        printf("client open unavailable rc=%d (%s)\n", rc, yai_sdk_errstr((yai_sdk_err_t)rc));
        return 0;
    }

    rc = yai_sdk_container_context_status(client, &reply);
    print_step("ws status", rc, &reply);
    yai_sdk_reply_free(&reply);

    rc = yai_sdk_container_graph_summary(client, &reply);
    print_step("ws graph summary", rc, &reply);
    yai_sdk_reply_free(&reply);

    rc = yai_sdk_container_db_status(client, &reply);
    print_step("ws db status", rc, &reply);
    yai_sdk_reply_free(&reply);

    rc = yai_sdk_container_data_evidence(client, &reply);
    print_step("ws data evidence", rc, &reply);
    yai_sdk_reply_free(&reply);

    rc = yai_sdk_container_cognition_status(client, &reply);
    print_step("ws cognition status", rc, &reply);
    yai_sdk_reply_free(&reply);

    rc = yai_sdk_container_policy_effective(client, &reply);
    print_step("ws policy effective", rc, &reply);
    yai_sdk_reply_free(&reply);

    rc = yai_sdk_container_debug_resolution(client, &reply);
    print_step("ws debug resolution", rc, &reply);
    yai_sdk_reply_free(&reply);

    yai_sdk_client_close(client);
    return 0;
}
