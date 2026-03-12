/* SPDX-License-Identifier: Apache-2.0 */

#include <assert.h>
#include <string.h>

#include "yai/shell/parse.h"

static void test_valid_runtime_ping(void)
{
    char *argv[] = {"yai", "runtime", "ping", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(3, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(req.command_id != NULL);
    assert(strcmp(req.command_id, "yai.core.ping") == 0);
    assert(req.error == NULL);
}

static void test_valid_verbose_contract(void)
{
    char *argv[] = {"yai", "--verbose-contract", "runtime", "ping", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(4, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(req.verbose_contract == 1);
    assert(strcmp(req.command_id, "yai.core.ping") == 0);
    assert(req.error == NULL);
}

static void test_builtin_lifecycle(void)
{
    char *argv[] = {"yai", "lifecycle", "up", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(3, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.lifecycle.up") == 0);
}

static void test_invalid_group(void)
{
    char *argv[] = {"yai", "wat", "ping", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(3, argv, &req);
    assert(rc != 0);
    assert(req.kind == YAI_PORCELAIN_KIND_ERROR);
    assert(req.error != NULL);
    assert(strcmp(req.error, "Unknown command group: wat") == 0);
    assert(req.error_hint != NULL);
    assert(strcmp(req.error_hint, "Run: yai help --groups") == 0);
}

static void test_help_group_command(void)
{
    char *argv[] = {"yai", "help", "runtime", "ping", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(4, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_HELP);
    assert(req.help_token != NULL);
    assert(strcmp(req.help_token, "runtime") == 0);
    assert(req.help_token2 != NULL);
    assert(strcmp(req.help_token2, "ping") == 0);
}

static void test_watch_parse(void)
{
    char *argv[] = {"yai", "watch", "runtime", "ping", "--count", "2", "--interval", "0.05", "--no-clear", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(9, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_WATCH);
    assert(req.cmd_argc == 7);
    assert(strcmp(req.cmd_argv[0], "runtime") == 0);
    assert(strcmp(req.cmd_argv[1], "ping") == 0);
    assert(req.watch_count == 2);
    assert(req.watch_interval_ms == 50);
    assert(req.watch_no_clear == 1);
}

static void test_watch_once_parse(void)
{
    char *argv[] = {"yai", "watch", "runtime", "ping", "--once", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(5, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_WATCH);
    assert(req.watch_count == 1);
}

static void test_group_without_command_prints_help(void)
{
    char *argv[] = {"yai", "runtime", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(2, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_HELP);
    assert(req.help_token != NULL);
    assert(strcmp(req.help_token, "runtime") == 0);
    assert(req.help_exit_code == 20);
}

static void test_color_mode_parse(void)
{
    char *argv[] = {"yai", "--color=never", "runtime", "ping", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(4, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(req.color_mode == YAI_COLOR_NEVER);
}

static void test_ws_current_maps_to_workspace_current(void)
{
    char *argv[] = {"yai", "ws", "current", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(3, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.container.current") == 0);
}

static void test_ws_status_maps_to_workspace_status(void)
{
    char *argv[] = {"yai", "ws", "status", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(3, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.container.status") == 0);
}

static void test_ws_domain_set_maps_to_workspace_domain_set(void)
{
    char *argv[] = {"yai", "ws", "domain", "set", "--family", "economic", "--specialization", "payments", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(8, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.container.domain_set") == 0);
}

static void test_ws_run_maps_to_workspace_run(void)
{
    char *argv[] = {"yai", "ws", "run", "payment.authorize", "provider=bank", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(5, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.container.run") == 0);
    assert(req.cmd_argc == 2);
    assert(strcmp(req.cmd_argv[0], "payment.authorize") == 0);
}

static void test_ws_run_missing_action_errors(void)
{
    char *argv[] = {"yai", "ws", "run", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(3, argv, &req);
    assert(rc != 0);
    assert(req.kind == YAI_PORCELAIN_KIND_ERROR);
    assert(req.error != NULL);
}

static void test_ws_prompt_token_maps_to_prompt_context(void)
{
    char *argv[] = {"yai", "ws", "prompt-token", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(3, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.container.prompt_context") == 0);
}

static void test_ws_graph_summary_maps_to_workspace_graph_summary(void)
{
    char *argv[] = {"yai", "ws", "graph", "summary", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(4, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.container.graph.summary") == 0);
}

static void test_ws_data_evidence_maps_to_workspace_query_family(void)
{
    char *argv[] = {"yai", "ws", "data", "evidence", "--limit", "5", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(6, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.container.data.evidence") == 0);
    assert(req.cmd_argc >= 1);
    assert(strcmp(req.cmd_argv[0], "evidence") == 0);
}

static void test_ws_db_tail_maps_to_workspace_events_tail(void)
{
    char *argv[] = {"yai", "ws", "db", "tail", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(4, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.container.db.tail") == 0);
}

static void test_ws_cognition_transient_maps_to_workspace_query(void)
{
    char *argv[] = {"yai", "ws", "cognition", "transient", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(4, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.container.cognition.transient") == 0);
    assert(strcmp(req.cmd_argv[0], "transient") == 0);
}

static void test_ws_recovery_reopen_maps_to_workspace_open(void)
{
    char *argv[] = {"yai", "ws", "recovery", "reopen", "prepilot_manual_01", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(5, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.container.open") == 0);
}

static void test_ws_query_maps_to_workspace_query(void)
{
    char *argv[] = {"yai", "ws", "query", "governance", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(4, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.container.query") == 0);
}

static void test_source_enroll_maps_to_source_enroll(void)
{
    char *argv[] = {"yai", "source", "enroll", "edge-a", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(4, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.source.enroll") == 0);
    assert(req.cmd_argc == 1);
    assert(strcmp(req.cmd_argv[0], "edge-a") == 0);
}

static void test_source_attach_maps_to_source_attach(void)
{
    char *argv[] = {"yai", "source", "attach", "sn-edge-a", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(4, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.source.attach") == 0);
}

static void test_source_list_maps_to_source_list(void)
{
    char *argv[] = {"yai", "source", "list", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(3, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.source.list") == 0);
}

static void test_source_status_maps_to_source_status(void)
{
    char *argv[] = {"yai", "source", "status", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(3, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.source.status") == 0);
}

static void test_source_inspect_maps_to_source_inspect(void)
{
    char *argv[] = {"yai", "source", "inspect", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(3, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.source.inspect") == 0);
}

static void test_source_retry_drain_maps_to_source_retry_drain(void)
{
    char *argv[] = {"yai", "source", "retry-drain", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(3, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.source.retry_drain") == 0);
}


static void test_inspect_source_projection_maps_to_source_inspect(void)
{
    char *argv[] = {"yai", "inspect", "source", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(3, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.source.inspect") == 0);
}

static void test_inspect_mesh_projection_maps_to_workspace_graph_workspace(void)
{
    char *argv[] = {"yai", "inspect", "mesh", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(3, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.container.graph.container") == 0);
}

static void test_inspect_grant_projection_maps_to_workspace_policy_effective(void)
{
    char *argv[] = {"yai", "inspect", "grant", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(3, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.container.policy_effective") == 0);
}

static void test_inspect_case_projection_maps_to_workspace_graph_summary(void)
{
    char *argv[] = {"yai", "inspect", "case", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(3, argv, &req);
    assert(rc == 0);
    assert(req.kind == YAI_PORCELAIN_KIND_COMMAND);
    assert(strcmp(req.command_id, "yai.container.graph.summary") == 0);
}

static void test_source_enroll_missing_label_errors(void)
{
    char *argv[] = {"yai", "source", "enroll", NULL};
    yai_porcelain_request_t req;
    int rc = yai_porcelain_parse_argv(3, argv, &req);
    assert(rc != 0);
    assert(req.kind == YAI_PORCELAIN_KIND_ERROR);
    assert(req.error != NULL);
}

int main(void)
{
    test_valid_runtime_ping();
    test_valid_verbose_contract();
    test_builtin_lifecycle();
    test_invalid_group();
    test_help_group_command();
    test_watch_parse();
    test_watch_once_parse();
    test_group_without_command_prints_help();
    test_color_mode_parse();
    test_ws_current_maps_to_workspace_current();
    test_ws_status_maps_to_workspace_status();
    test_ws_domain_set_maps_to_workspace_domain_set();
    test_ws_run_maps_to_workspace_run();
    test_ws_run_missing_action_errors();
    test_ws_prompt_token_maps_to_prompt_context();
    test_ws_graph_summary_maps_to_workspace_graph_summary();
    test_ws_data_evidence_maps_to_workspace_query_family();
    test_ws_db_tail_maps_to_workspace_events_tail();
    test_ws_cognition_transient_maps_to_workspace_query();
    test_ws_recovery_reopen_maps_to_workspace_open();
    test_ws_query_maps_to_workspace_query();
    test_source_enroll_maps_to_source_enroll();
    test_source_attach_maps_to_source_attach();
    test_source_list_maps_to_source_list();
    test_source_status_maps_to_source_status();
    test_source_inspect_maps_to_source_inspect();
    test_source_retry_drain_maps_to_source_retry_drain();
    test_inspect_source_projection_maps_to_source_inspect();
    test_inspect_mesh_projection_maps_to_workspace_graph_workspace();
    test_inspect_grant_projection_maps_to_workspace_policy_effective();
    test_inspect_case_projection_maps_to_workspace_graph_summary();
    test_source_enroll_missing_label_errors();
    return 0;
}
