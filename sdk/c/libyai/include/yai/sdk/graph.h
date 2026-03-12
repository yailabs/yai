/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <string.h>
#include <yai/sdk/container.h>

#ifdef __cplusplus
extern "C" {
#endif

#define YAI_SDK_GRAPH_QUERY_FAMILY "graph"
#define YAI_SDK_GRAPH_CMD_SUMMARY YAI_SDK_CMD_CONTAINER_GRAPH_SUMMARY
#define YAI_SDK_GRAPH_CMD_WORKSPACE YAI_SDK_CMD_CONTAINER_GRAPH_WORKSPACE
#define YAI_SDK_GRAPH_CMD_GOVERNANCE YAI_SDK_CMD_CONTAINER_GRAPH_GOVERNANCE
#define YAI_SDK_GRAPH_CMD_DECISION YAI_SDK_CMD_CONTAINER_GRAPH_DECISION
#define YAI_SDK_GRAPH_CMD_EVIDENCE YAI_SDK_CMD_CONTAINER_GRAPH_EVIDENCE
#define YAI_SDK_GRAPH_CMD_AUTHORITY YAI_SDK_CMD_CONTAINER_GRAPH_AUTHORITY
#define YAI_SDK_GRAPH_CMD_ARTIFACT YAI_SDK_CMD_CONTAINER_GRAPH_ARTIFACT
#define YAI_SDK_GRAPH_CMD_LINEAGE YAI_SDK_CMD_CONTAINER_GRAPH_LINEAGE
#define YAI_SDK_GRAPH_CMD_RECENT YAI_SDK_CMD_CONTAINER_GRAPH_RECENT
#define YAI_SDK_GRAPH_CMD_QUERY YAI_SDK_CMD_CONTAINER_QUERY

static inline int yai_sdk_graph_is_query_family(const char *query_family)
{
    return query_family && strcmp(query_family, YAI_SDK_GRAPH_QUERY_FAMILY) == 0;
}

static inline int yai_sdk_container_graph_summary(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_GRAPH_CMD_SUMMARY, out);
}

static inline int yai_sdk_container_graph_container(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_GRAPH_CMD_WORKSPACE, out);
}

static inline int yai_sdk_container_graph_governance(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_GRAPH_CMD_GOVERNANCE, out);
}

static inline int yai_sdk_container_graph_decision(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_GRAPH_CMD_DECISION, out);
}

static inline int yai_sdk_container_graph_evidence(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_GRAPH_CMD_EVIDENCE, out);
}

static inline int yai_sdk_container_graph_authority(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_GRAPH_CMD_AUTHORITY, out);
}

static inline int yai_sdk_container_graph_artifact(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_GRAPH_CMD_ARTIFACT, out);
}

static inline int yai_sdk_container_graph_lineage(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_GRAPH_CMD_LINEAGE, out);
}

static inline int yai_sdk_container_graph_recent(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_GRAPH_CMD_RECENT, out);
}

#ifdef __cplusplus
}
#endif
