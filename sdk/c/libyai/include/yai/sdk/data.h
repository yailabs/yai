/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <string.h>
#include <yai/sdk/container.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Canonical data-plane query families. */
#define YAI_SDK_DATA_QUERY_EVENTS "events"
#define YAI_SDK_DATA_QUERY_ENFORCEMENT "enforcement"
#define YAI_SDK_DATA_QUERY_AUTHORITY "authority"
#define YAI_SDK_DATA_QUERY_GOVERNANCE "governance"
#define YAI_SDK_DATA_QUERY_EVIDENCE "evidence"
#define YAI_SDK_DATA_QUERY_ARTIFACT "artifact"

/* Data-plane reads are exposed through container query. */
#define YAI_SDK_DATA_CMD_QUERY YAI_SDK_CMD_CONTAINER_QUERY

static inline int yai_sdk_data_is_query_family(const char *query_family)
{
    if (!query_family) {
        return 0;
    }
    return strcmp(query_family, YAI_SDK_DATA_QUERY_EVENTS) == 0 ||
           strcmp(query_family, YAI_SDK_DATA_QUERY_ENFORCEMENT) == 0 ||
           strcmp(query_family, YAI_SDK_DATA_QUERY_AUTHORITY) == 0 ||
           strcmp(query_family, YAI_SDK_DATA_QUERY_GOVERNANCE) == 0 ||
           strcmp(query_family, YAI_SDK_DATA_QUERY_EVIDENCE) == 0 ||
           strcmp(query_family, YAI_SDK_DATA_QUERY_ARTIFACT) == 0;
}

static inline int yai_sdk_container_data_events(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_query_family(client, YAI_SDK_DATA_QUERY_EVENTS, out);
}

static inline int yai_sdk_container_data_evidence(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_query_family(client, YAI_SDK_DATA_QUERY_EVIDENCE, out);
}

static inline int yai_sdk_container_data_governance(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_query_family(client, YAI_SDK_DATA_QUERY_GOVERNANCE, out);
}

static inline int yai_sdk_container_data_authority(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_query_family(client, YAI_SDK_DATA_QUERY_AUTHORITY, out);
}

static inline int yai_sdk_container_data_artifacts(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_query_family(client, YAI_SDK_DATA_QUERY_ARTIFACT, out);
}

static inline int yai_sdk_container_data_enforcement(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_query_family(client, YAI_SDK_DATA_QUERY_ENFORCEMENT, out);
}

#ifdef __cplusplus
}
#endif
