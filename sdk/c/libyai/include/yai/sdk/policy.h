/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/sdk/container.h>

#ifdef __cplusplus
extern "C" {
#endif

#define YAI_SDK_POLICY_CMD_ATTACH YAI_SDK_CMD_CONTAINER_POLICY_ATTACH
#define YAI_SDK_POLICY_CMD_DETACH YAI_SDK_CMD_CONTAINER_POLICY_DETACH
#define YAI_SDK_POLICY_CMD_ACTIVATE YAI_SDK_CMD_CONTAINER_POLICY_ACTIVATE
#define YAI_SDK_POLICY_CMD_DRY_RUN YAI_SDK_CMD_CONTAINER_POLICY_DRY_RUN
#define YAI_SDK_POLICY_CMD_EFFECTIVE YAI_SDK_CMD_CONTAINER_POLICY_EFFECTIVE

static inline int yai_sdk_container_policy_attach_object(
    yai_sdk_client_t *client,
    const char *object_id,
    yai_sdk_reply_t *out)
{
    return yai_sdk_container_policy_attach(client, object_id, out);
}

static inline int yai_sdk_container_policy_detach_object(
    yai_sdk_client_t *client,
    const char *object_id,
    yai_sdk_reply_t *out)
{
    return yai_sdk_container_policy_detach(client, object_id, out);
}

static inline int yai_sdk_container_policy_activate_object(
    yai_sdk_client_t *client,
    const char *object_id,
    yai_sdk_reply_t *out)
{
    return yai_sdk_container_policy_activate(client, object_id, out);
}

static inline int yai_sdk_container_policy_dry_run_object(
    yai_sdk_client_t *client,
    const char *object_id,
    yai_sdk_reply_t *out)
{
    return yai_sdk_container_policy_dry_run(client, object_id, out);
}

#ifdef __cplusplus
}
#endif
