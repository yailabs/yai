/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/sdk/client.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Canonical runtime control-call envelope type. */
#define YAI_SDK_RUNTIME_CONTROL_CALL_TYPE "yai.control.call.v1"

/* Canonical runtime target plane name. */
#define YAI_SDK_RUNTIME_TARGET_PLANE "runtime"

/* Canonical runtime command ids exposed to SDK consumers. */
#define YAI_SDK_CMD_RUNTIME_PING "yai.runtime.ping"
#define YAI_SDK_CMD_RUNTIME_SCOPE_STATUS "yai.runtime.scope_status"

/* Runtime ping over canonical runtime surface. */
static inline int yai_sdk_runtime_ping(
    yai_sdk_client_t *client,
    yai_sdk_reply_t *out)
{
    return yai_sdk_client_ping(client, YAI_SDK_CMD_RUNTIME_PING, out);
}

#ifdef __cplusplus
}
#endif
