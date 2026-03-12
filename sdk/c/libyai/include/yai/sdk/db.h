/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/sdk/container.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Workspace-first DB surface (direct + composition-backed). */
#define YAI_SDK_DB_CMD_STATUS YAI_SDK_CMD_CONTAINER_STATUS
#define YAI_SDK_DB_CMD_BINDINGS YAI_SDK_CMD_CONTAINER_INSPECT
#define YAI_SDK_DB_CMD_STORES YAI_SDK_CMD_CONTAINER_INSPECT
#define YAI_SDK_DB_CMD_CLASSES YAI_SDK_CMD_CONTAINER_QUERY
#define YAI_SDK_DB_CMD_COUNT YAI_SDK_CMD_CONTAINER_QUERY
#define YAI_SDK_DB_CMD_TAIL YAI_SDK_CMD_CONTAINER_EVENTS_TAIL

static inline int yai_sdk_container_db_status(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_DB_CMD_STATUS, out);
}

static inline int yai_sdk_container_db_bindings(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_DB_CMD_BINDINGS, out);
}

static inline int yai_sdk_container_db_stores(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_DB_CMD_STORES, out);
}

static inline int yai_sdk_container_db_classes(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_query_family(client, YAI_SDK_CONTAINER_QUERY_FAMILY_WORKSPACE, out);
}

static inline int yai_sdk_container_db_count(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_query_family(client, YAI_SDK_CONTAINER_QUERY_FAMILY_EVENTS, out);
}

static inline int yai_sdk_container_db_tail(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_DB_CMD_TAIL, out);
}

#ifdef __cplusplus
}
#endif
