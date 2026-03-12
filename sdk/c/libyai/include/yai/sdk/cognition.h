/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <string.h>
#include <yai/sdk/container.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Canonical cognition-support query families. */
#define YAI_SDK_KNOWLEDGE_QUERY_TRANSIENT "transient"
#define YAI_SDK_KNOWLEDGE_QUERY_MEMORY "memory"
#define YAI_SDK_KNOWLEDGE_QUERY_PROVIDERS "providers"
#define YAI_SDK_KNOWLEDGE_QUERY_CONTEXT "context"

#define YAI_SDK_KNOWLEDGE_CMD_QUERY YAI_SDK_CMD_CONTAINER_QUERY
#define YAI_SDK_KNOWLEDGE_CMD_STATUS YAI_SDK_CMD_CONTAINER_INSPECT

static inline int yai_sdk_cognition_is_query_family(const char *query_family)
{
    if (!query_family) {
        return 0;
    }
    return strcmp(query_family, YAI_SDK_KNOWLEDGE_QUERY_TRANSIENT) == 0 ||
           strcmp(query_family, YAI_SDK_KNOWLEDGE_QUERY_MEMORY) == 0 ||
           strcmp(query_family, YAI_SDK_KNOWLEDGE_QUERY_PROVIDERS) == 0 ||
           strcmp(query_family, YAI_SDK_KNOWLEDGE_QUERY_CONTEXT) == 0;
}

static inline int yai_sdk_container_cognition_status(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_KNOWLEDGE_CMD_STATUS, out);
}

static inline int yai_sdk_container_cognition_transient(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_query_family(client, YAI_SDK_KNOWLEDGE_QUERY_TRANSIENT, out);
}

static inline int yai_sdk_container_cognition_memory(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_query_family(client, YAI_SDK_KNOWLEDGE_QUERY_MEMORY, out);
}

static inline int yai_sdk_container_cognition_providers(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_query_family(client, YAI_SDK_KNOWLEDGE_QUERY_PROVIDERS, out);
}

static inline int yai_sdk_container_cognition_context(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_query_family(client, YAI_SDK_KNOWLEDGE_QUERY_CONTEXT, out);
}

#ifdef __cplusplus
}
#endif
