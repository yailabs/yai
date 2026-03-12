/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/sdk/container.h>

#ifdef __cplusplus
extern "C" {
#endif

#define YAI_SDK_RECOVERY_CMD_STATUS YAI_SDK_CMD_CONTAINER_RECOVERY_STATUS
#define YAI_SDK_RECOVERY_CMD_LOAD YAI_SDK_CMD_CONTAINER_RECOVERY_LOAD
#define YAI_SDK_RECOVERY_CMD_REOPEN YAI_SDK_CMD_CONTAINER_RECOVERY_REOPEN

static inline int yai_sdk_container_recovery_reopen_current(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    char ws_id[64] = {0};
    if (yai_sdk_container_context_current(ws_id, sizeof(ws_id)) != 0 || !ws_id[0]) {
        return YAI_SDK_BAD_ARGS;
    }
    return yai_sdk_container_recovery_reopen(client, ws_id, out);
}

#ifdef __cplusplus
}
#endif
