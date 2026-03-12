// SPDX-License-Identifier: Apache-2.0
#include <stdio.h>
#include <string.h>

#include "yai/sdk/sdk.h"

int main(void)
{
  yai_sdk_client_t *client = NULL;
  yai_sdk_client_opts_t opts = {0};
  yai_sdk_runtime_endpoint_t endpoint;
  yai_sdk_reply_t out = {0};
  yai_sdk_source_enroll_request_t enroll_req = {
      .workspace_id = "ws_source_smoke",
      .source_label = "node-a",
      .owner_ref = "uds:///tmp/yai-owner.sock",
  };
  yai_sdk_source_attach_request_t attach_req = {
      .workspace_id = "ws_source_smoke",
      .source_node_id = "sn-node-a",
      .binding_scope = "container",
  };
  yai_sdk_source_status_request_t status_req = {
      .workspace_id = "ws_source_smoke",
      .source_node_id = "sn-node-a",
      .daemon_instance_id = "sd-node-a",
      .health = "ready",
  };
  yai_sdk_source_emit_request_t emit_req = {
      .workspace_id = "ws_source_smoke",
      .source_node_id = "sn-node-a",
      .source_binding_id = "sb-node-a",
      .idempotency_key = "k1",
  };
  yai_sdk_source_enroll_reply_t enroll_out;
  yai_sdk_source_attach_reply_t attach_out;
  yai_sdk_source_emit_reply_t emit_out;
  yai_sdk_source_status_reply_t status_out;
  yai_sdk_source_summary_t summary_out;
  yai_sdk_reply_t fake_reply = {0};

  if (!yai_sdk_source_is_query_family("source")) {
    fprintf(stderr, "source_typed_surface_smoke: source query family guard mismatch\n");
    return 1;
  }
  if (strcmp(YAI_SDK_SOURCE_CMD_ENROLL, "yai.source.enroll") != 0 ||
      strcmp(YAI_SDK_SOURCE_CMD_ATTACH, "yai.source.attach") != 0 ||
      strcmp(YAI_SDK_SOURCE_CMD_EMIT, "yai.source.emit") != 0 ||
      strcmp(YAI_SDK_SOURCE_CMD_STATUS, "yai.source.status") != 0) {
    fprintf(stderr, "source_typed_surface_smoke: command constants mismatch\n");
    return 2;
  }

  if (yai_sdk_source_enroll(NULL, &enroll_req, &out) != YAI_SDK_BAD_ARGS) return 3;
  if (yai_sdk_source_attach(NULL, &attach_req, &out) != YAI_SDK_BAD_ARGS) return 4;
  if (yai_sdk_source_emit(NULL, &emit_req, &out) != YAI_SDK_BAD_ARGS) return 5;
  if (yai_sdk_source_status(NULL, &status_req, &out) != YAI_SDK_BAD_ARGS) return 6;
  if (yai_sdk_source_summary(NULL, &out) != YAI_SDK_BAD_ARGS) return 7;

  fake_reply.exec_reply_json =
      "{\"type\":\"yai.exec.reply.v1\",\"status\":\"ok\",\"code\":\"OK\",\"reason\":\"source_emit_accepted\","
      "\"data\":{\"workspace_id\":\"ws_source_smoke\",\"source_node_id\":\"sn-node-a\",\"source_binding_id\":\"sb-node-a\","
      "\"idempotency_key\":\"k1\",\"accepted\":{\"source_asset\":1,\"source_acquisition_event\":2,\"source_evidence_candidate\":3},"
      "\"mediation\":{\"layer\":\"exec\",\"stage\":\"exec.runtime.source_plane_mediation.v1\",\"topology\":\"distributed-acquisition-centralized-control-v1\","
      "\"route\":\"owner_ingest_v1\",\"owner_canonical\":true,\"transport_ready\":true,\"network_gate_ready\":true,\"resource_gate_ready\":true,\"storage_gate_ready\":true}},"
      "\"command_id\":\"yai.source.emit\",\"target_plane\":\"runtime\"}";

  if (yai_sdk_source_emit_reply_from_sdk(&fake_reply, &emit_out) != YAI_SDK_OK) return 8;
  if (emit_out.accepted_assets != 1 || emit_out.accepted_events != 2 || emit_out.accepted_candidates != 3) return 9;
  if (strcmp(emit_out.mediation.layer, "exec") != 0 || strcmp(emit_out.mediation.route, "owner_ingest_v1") != 0) return 10;

  fake_reply.exec_reply_json =
      "{\"type\":\"yai.exec.reply.v1\",\"status\":\"ok\",\"code\":\"OK\",\"reason\":\"workspace_query_result\","
      "\"data\":{\"type\":\"yai.container.query.result.v1\",\"query_family\":\"source\","
      "\"summary\":{\"source_node_count\":1,\"source_daemon_instance_count\":1,\"source_binding_count\":1,"
      "\"source_asset_count\":2,\"source_acquisition_event_count\":3,\"source_evidence_candidate_count\":4,"
      "\"source_owner_link_count\":1,\"source_graph_node_count\":5,\"source_graph_edge_count\":6}}}";

  if (yai_sdk_source_summary_from_sdk(&fake_reply, &summary_out) != YAI_SDK_OK) return 11;
  if (summary_out.source_graph_node_count != 5 || summary_out.source_graph_edge_count != 6) return 12;

  yai_sdk_source_enroll_reply_init(&enroll_out);
  yai_sdk_source_attach_reply_init(&attach_out);
  yai_sdk_source_status_reply_init(&status_out);
  if (enroll_out.registered != 0 || attach_out.attachment_status[0] != '\0' || status_out.health[0] != '\0') return 13;

  /* Source typed APIs run over the same endpoint-aware client model (YDS-2). */
  if (yai_sdk_runtime_endpoint_owner_ref("uds:///tmp/yai-source-owner.sock", &endpoint) != 0) return 14;
  opts.ws_id = "ws_source_smoke";
  opts.runtime_endpoint = &endpoint;
  if (yai_sdk_client_open(&client, &opts) != YAI_SDK_SERVER_OFF) return 15;

  puts("source_typed_surface_smoke: ok");
  return 0;
}
