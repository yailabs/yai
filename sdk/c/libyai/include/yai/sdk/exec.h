/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/sdk/container.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Execution-facing runtime commands (exec family). */
#define YAI_SDK_EXEC_CMD_RUN_ACTION YAI_SDK_CMD_CONTAINER_RUN
#define YAI_SDK_EXEC_CMD_POLICY_DRY_RUN YAI_SDK_CMD_CONTAINER_POLICY_DRY_RUN
#define YAI_SDK_EXEC_CMD_POLICY_ATTACH YAI_SDK_CMD_CONTAINER_POLICY_ATTACH
#define YAI_SDK_EXEC_CMD_POLICY_ACTIVATE YAI_SDK_CMD_CONTAINER_POLICY_ACTIVATE
#define YAI_SDK_EXEC_CMD_POLICY_DETACH YAI_SDK_CMD_CONTAINER_POLICY_DETACH

static inline int yai_sdk_container_run_action(
    yai_sdk_client_t *client,
    const char *action_id,
    yai_sdk_reply_t *out)
{
    return yai_sdk_container_command1(client, YAI_SDK_EXEC_CMD_RUN_ACTION, action_id, out);
}

#ifdef __cplusplus
}
#endif
