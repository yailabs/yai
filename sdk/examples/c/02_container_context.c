/* SPDX-License-Identifier: Apache-2.0 */
#include <stdio.h>

#include <yai/sdk/sdk.h>

int main(void)
{
    char ws_id[64] = {0};
    yai_sdk_container_info_t info = {0};
    int rc = 0;

    rc = yai_sdk_container_context_bind("ws_demo");
    if (rc != 0) {
        fprintf(stderr, "container_bind failed\n");
        return 1;
    }

    rc = yai_sdk_container_context_current(ws_id, sizeof(ws_id));
    if (rc != 0) {
        fprintf(stderr, "container_current failed (%d)\n", rc);
        return 2;
    }

    printf("current container: %s\n", ws_id);

    rc = yai_sdk_container_context_validate_binding(&info);
    if (rc == YAI_SDK_OK) {
        printf("runtime status: exists=%d state=%s root=%s\n",
               info.exists,
               info.state,
               info.root_path);
    } else {
        printf("runtime validation not available: %s\n", yai_sdk_errstr((yai_sdk_err_t)rc));
    }

    (void)yai_sdk_container_context_clear_binding();
    return 0;
}
