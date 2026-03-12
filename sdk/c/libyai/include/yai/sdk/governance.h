/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/sdk/policy.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Governance/law-adjacent runtime command ids. */
#define YAI_SDK_GOV_CMD_POLICY_DRY_RUN YAI_SDK_CMD_CONTAINER_POLICY_DRY_RUN
#define YAI_SDK_GOV_CMD_POLICY_ATTACH YAI_SDK_CMD_CONTAINER_POLICY_ATTACH
#define YAI_SDK_GOV_CMD_POLICY_ACTIVATE YAI_SDK_CMD_CONTAINER_POLICY_ACTIVATE
#define YAI_SDK_GOV_CMD_POLICY_DETACH YAI_SDK_CMD_CONTAINER_POLICY_DETACH
#define YAI_SDK_GOV_CMD_POLICY_EFFECTIVE YAI_SDK_CMD_CONTAINER_POLICY_EFFECTIVE

/* Governance-facing typed aliases over container policy surface. */
static inline int yai_sdk_gov_policy_dry_run(
    yai_sdk_client_t *client,
    const char *object_id,
    yai_sdk_reply_t *out)
{
    return yai_sdk_container_policy_dry_run_object(client, object_id, out);
}

static inline int yai_sdk_gov_policy_attach(
    yai_sdk_client_t *client,
    const char *object_id,
    yai_sdk_reply_t *out)
{
    return yai_sdk_container_policy_attach_object(client, object_id, out);
}

static inline int yai_sdk_gov_policy_activate(
    yai_sdk_client_t *client,
    const char *object_id,
    yai_sdk_reply_t *out)
{
    return yai_sdk_container_policy_activate_object(client, object_id, out);
}

static inline int yai_sdk_gov_policy_detach(
    yai_sdk_client_t *client,
    const char *object_id,
    yai_sdk_reply_t *out)
{
    return yai_sdk_container_policy_detach_object(client, object_id, out);
}

static inline int yai_sdk_gov_policy_effective(
    yai_sdk_client_t *client,
    yai_sdk_reply_t *out)
{
    return yai_sdk_container_policy_effective(client, out);
}

#ifdef __cplusplus
}
#endif
