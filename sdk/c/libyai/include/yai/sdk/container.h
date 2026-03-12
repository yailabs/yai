/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/sdk/context.h>
#include <yai/sdk/client.h>
#include <yai/sdk/errors.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Canonical container command ids for container-bound runtime interaction. */
#define YAI_SDK_CMD_CONTAINER_CREATE "yai.container.create"
#define YAI_SDK_CMD_CONTAINER_OPEN "yai.container.open"
#define YAI_SDK_CMD_CONTAINER_SET "yai.container.set"
#define YAI_SDK_CMD_CONTAINER_SWITCH "yai.container.switch"
#define YAI_SDK_CMD_CONTAINER_CURRENT "yai.container.current"
#define YAI_SDK_CMD_CONTAINER_UNSET "yai.container.unset"
#define YAI_SDK_CMD_CONTAINER_CLEAR "yai.container.clear"
#define YAI_SDK_CMD_CONTAINER_RESET "yai.container.reset"
#define YAI_SDK_CMD_CONTAINER_DESTROY "yai.container.destroy"
#define YAI_SDK_CMD_CONTAINER_STATUS "yai.container.status"
#define YAI_SDK_CMD_CONTAINER_INSPECT "yai.container.inspect"
#define YAI_SDK_CMD_CONTAINER_QUERY "yai.container.query"
#define YAI_SDK_CMD_CONTAINER_GRAPH_SUMMARY "yai.container.graph.summary"
#define YAI_SDK_CMD_CONTAINER_GRAPH_WORKSPACE "yai.container.graph.workspace"
#define YAI_SDK_CMD_CONTAINER_GRAPH_GOVERNANCE "yai.container.graph.governance"
#define YAI_SDK_CMD_CONTAINER_GRAPH_DECISION "yai.container.graph.decision"
#define YAI_SDK_CMD_CONTAINER_GRAPH_EVIDENCE "yai.container.graph.evidence"
#define YAI_SDK_CMD_CONTAINER_GRAPH_AUTHORITY "yai.container.graph.authority"
#define YAI_SDK_CMD_CONTAINER_GRAPH_ARTIFACT "yai.container.graph.artifact"
#define YAI_SDK_CMD_CONTAINER_GRAPH_LINEAGE "yai.container.graph.lineage"
#define YAI_SDK_CMD_CONTAINER_GRAPH_RECENT "yai.container.graph.recent"
#define YAI_SDK_CMD_CONTAINER_RUN "yai.container.run"
#define YAI_SDK_CMD_CONTAINER_EVENTS_TAIL "yai.container.events.tail"
#define YAI_SDK_CMD_CONTAINER_POLICY_EFFECTIVE "yai.container.policy_effective"
#define YAI_SDK_CMD_CONTAINER_POLICY_DRY_RUN "yai.container.policy_dry_run"
#define YAI_SDK_CMD_CONTAINER_POLICY_ATTACH "yai.container.policy_attach"
#define YAI_SDK_CMD_CONTAINER_POLICY_ACTIVATE "yai.container.policy_activate"
#define YAI_SDK_CMD_CONTAINER_POLICY_DETACH "yai.container.policy_detach"
#define YAI_SDK_CMD_CONTAINER_DOMAIN_GET "yai.container.domain_get"
#define YAI_SDK_CMD_CONTAINER_DOMAIN_SET "yai.container.domain_set"
#define YAI_SDK_CMD_CONTAINER_DEBUG_RESOLUTION "yai.container.debug_resolution"
#define YAI_SDK_CMD_CONTAINER_RECOVERY_STATUS "yai.container.status"
#define YAI_SDK_CMD_CONTAINER_RECOVERY_LOAD "yai.container.lifecycle.maintain"
#define YAI_SDK_CMD_CONTAINER_RECOVERY_REOPEN "yai.container.open"
#define YAI_SDK_CMD_SOURCE_ENROLL "yai.source.enroll"
#define YAI_SDK_CMD_SOURCE_ATTACH "yai.source.attach"
#define YAI_SDK_CMD_SOURCE_EMIT "yai.source.emit"
#define YAI_SDK_CMD_SOURCE_STATUS "yai.source.status"

/* Container query families surfaced by canonical typed helpers. */
#define YAI_SDK_CONTAINER_QUERY_FAMILY_WORKSPACE "workspace"
#define YAI_SDK_CONTAINER_QUERY_FAMILY_EVENTS "events"
#define YAI_SDK_CONTAINER_QUERY_FAMILY_EVIDENCE "evidence"
#define YAI_SDK_CONTAINER_QUERY_FAMILY_GOVERNANCE "governance"
#define YAI_SDK_CONTAINER_QUERY_FAMILY_AUTHORITY "authority"
#define YAI_SDK_CONTAINER_QUERY_FAMILY_ARTIFACT "artifact"
#define YAI_SDK_CONTAINER_QUERY_FAMILY_ENFORCEMENT "enforcement"
#define YAI_SDK_CONTAINER_QUERY_FAMILY_TRANSIENT "transient"
#define YAI_SDK_CONTAINER_QUERY_FAMILY_MEMORY "memory"
#define YAI_SDK_CONTAINER_QUERY_FAMILY_PROVIDERS "providers"
#define YAI_SDK_CONTAINER_QUERY_FAMILY_CONTEXT "context"
#define YAI_SDK_CONTAINER_QUERY_FAMILY_SOURCE "source"

static inline int yai_sdk_container_command(
    yai_sdk_client_t *client,
    const char *command_id,
    size_t argv_len,
    const char *const *argv,
    yai_sdk_reply_t *out)
{
    yai_sdk_control_call_t call = {
        .target_plane = "runtime",
        .command_id = command_id,
        .argv = argv,
        .argv_len = argv_len,
    };
    return yai_sdk_client_call(client, &call, out);
}

static inline int yai_sdk_container_command0(
    yai_sdk_client_t *client,
    const char *command_id,
    yai_sdk_reply_t *out)
{
    return yai_sdk_container_command(client, command_id, 0, NULL, out);
}

static inline int yai_sdk_container_command1(
    yai_sdk_client_t *client,
    const char *command_id,
    const char *arg0,
    yai_sdk_reply_t *out)
{
    const char *argv[1] = {arg0};
    return yai_sdk_container_command(client, command_id, 1, argv, out);
}

/* Canonical container-first runtime helpers (lifecycle/binding). */
static inline int yai_sdk_container_create(yai_sdk_client_t *client, const char *container_id, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command1(client, YAI_SDK_CMD_CONTAINER_CREATE, container_id, out);
}

static inline int yai_sdk_container_open(yai_sdk_client_t *client, const char *container_id, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command1(client, YAI_SDK_CMD_CONTAINER_OPEN, container_id, out);
}

static inline int yai_sdk_container_set(yai_sdk_client_t *client, const char *container_id, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command1(client, YAI_SDK_CMD_CONTAINER_SET, container_id, out);
}

static inline int yai_sdk_container_switch(yai_sdk_client_t *client, const char *container_id, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command1(client, YAI_SDK_CMD_CONTAINER_SWITCH, container_id, out);
}

static inline int yai_sdk_container_current(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_CMD_CONTAINER_CURRENT, out);
}

static inline int yai_sdk_container_status(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_CMD_CONTAINER_STATUS, out);
}

static inline int yai_sdk_container_inspect(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_CMD_CONTAINER_INSPECT, out);
}

static inline int yai_sdk_container_unset(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_CMD_CONTAINER_UNSET, out);
}

static inline int yai_sdk_container_clear(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_CMD_CONTAINER_CLEAR, out);
}

static inline int yai_sdk_container_reset(yai_sdk_client_t *client, const char *container_id, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command1(client, YAI_SDK_CMD_CONTAINER_RESET, container_id, out);
}

static inline int yai_sdk_container_destroy(yai_sdk_client_t *client, const char *container_id, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command1(client, YAI_SDK_CMD_CONTAINER_DESTROY, container_id, out);
}

/* Generic query fallback stays available by design. */
static inline int yai_sdk_container_query_family(
    yai_sdk_client_t *client,
    const char *query_family,
    yai_sdk_reply_t *out)
{
    return yai_sdk_container_command1(client, YAI_SDK_CMD_CONTAINER_QUERY, query_family, out);
}

/* Domain helpers. */
static inline int yai_sdk_container_domain_get(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_CMD_CONTAINER_DOMAIN_GET, out);
}

static inline int yai_sdk_container_domain_set(
    yai_sdk_client_t *client,
    const char *family,
    const char *specialization,
    yai_sdk_reply_t *out)
{
    if (!family || !family[0] || !specialization || !specialization[0]) {
        return YAI_SDK_BAD_ARGS;
    }
    const char *argv[4] = {"--family", family, "--specialization", specialization};
    return yai_sdk_container_command(client, YAI_SDK_CMD_CONTAINER_DOMAIN_SET, 4, argv, out);
}

/* Policy/governance helpers. */
static inline int yai_sdk_container_policy_attach(
    yai_sdk_client_t *client,
    const char *object_id,
    yai_sdk_reply_t *out)
{
    if (!object_id || !object_id[0]) {
        return YAI_SDK_BAD_ARGS;
    }
    return yai_sdk_container_command1(client, YAI_SDK_CMD_CONTAINER_POLICY_ATTACH, object_id, out);
}

static inline int yai_sdk_container_policy_detach(
    yai_sdk_client_t *client,
    const char *object_id,
    yai_sdk_reply_t *out)
{
    if (!object_id || !object_id[0]) {
        return YAI_SDK_BAD_ARGS;
    }
    return yai_sdk_container_command1(client, YAI_SDK_CMD_CONTAINER_POLICY_DETACH, object_id, out);
}

static inline int yai_sdk_container_policy_activate(
    yai_sdk_client_t *client,
    const char *object_id,
    yai_sdk_reply_t *out)
{
    if (!object_id || !object_id[0]) {
        return YAI_SDK_BAD_ARGS;
    }
    return yai_sdk_container_command1(client, YAI_SDK_CMD_CONTAINER_POLICY_ACTIVATE, object_id, out);
}

static inline int yai_sdk_container_policy_dry_run(
    yai_sdk_client_t *client,
    const char *object_id,
    yai_sdk_reply_t *out)
{
    if (!object_id || !object_id[0]) {
        return YAI_SDK_BAD_ARGS;
    }
    return yai_sdk_container_command1(client, YAI_SDK_CMD_CONTAINER_POLICY_DRY_RUN, object_id, out);
}

static inline int yai_sdk_container_policy_effective(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_CMD_CONTAINER_POLICY_EFFECTIVE, out);
}

/* Recovery/debug helpers. */
static inline int yai_sdk_container_recovery_status(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_CMD_CONTAINER_RECOVERY_STATUS, out);
}

static inline int yai_sdk_container_recovery_load(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_CMD_CONTAINER_RECOVERY_LOAD, out);
}

static inline int yai_sdk_container_recovery_reopen(
    yai_sdk_client_t *client,
    const char *container_id,
    yai_sdk_reply_t *out)
{
    if (!container_id || !container_id[0]) {
        return YAI_SDK_BAD_ARGS;
    }
    return yai_sdk_container_command1(client, YAI_SDK_CMD_CONTAINER_RECOVERY_REOPEN, container_id, out);
}

static inline int yai_sdk_container_debug_resolution(yai_sdk_client_t *client, yai_sdk_reply_t *out)
{
    return yai_sdk_container_command0(client, YAI_SDK_CMD_CONTAINER_DEBUG_RESOLUTION, out);
}

/* Canonical container-binding API aliases. */
static inline int yai_sdk_container_context_bind(const char *container_id)
{
    return yai_sdk_context_set_current_container(container_id);
}

static inline int yai_sdk_container_context_switch(const char *container_id)
{
    return yai_sdk_context_switch_container(container_id);
}

static inline int yai_sdk_container_context_unbind(void)
{
    return yai_sdk_context_unset_container();
}

static inline int yai_sdk_container_context_clear_binding(void)
{
    return yai_sdk_context_clear_current_container();
}

static inline int yai_sdk_container_context_current(char *out_container_id, size_t out_cap)
{
    return yai_sdk_context_get_current_container(out_container_id, out_cap);
}

static inline int yai_sdk_container_context_resolve(
    const char *explicit_container_id,
    char *out_container_id,
    size_t out_cap)
{
    return yai_sdk_context_resolve_container(explicit_container_id, out_container_id, out_cap);
}

/* Runtime-backed container status helper (thin alias). */
static inline int yai_sdk_container_context_status(
    const char *container_id,
    yai_sdk_container_info_t *out)
{
    return yai_sdk_container_describe(container_id, out);
}

static inline int yai_sdk_container_context_validate_binding(yai_sdk_container_info_t *out)
{
    return yai_sdk_context_validate_current_container(out);
}

#ifdef __cplusplus
}
#endif
