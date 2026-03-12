// SPDX-License-Identifier: Apache-2.0

#include <stdio.h>
#include <string.h>

#include "yai/sdk/sdk.h"

int main(void)
{
  static const char *argvv[] = {"--family", "enforcement"};
  yai_sdk_control_call_t call = {
      .target_plane = YAI_SDK_RUNTIME_TARGET_PLANE,
      .command_id = YAI_SDK_DATA_CMD_QUERY,
      .argv = argvv,
      .argv_len = 2,
  };
  char call_json[256];

  static const char *reply_json =
      "{"
      "\"type\":\"yai.exec.reply.v1\"," 
      "\"status\":\"ok\"," 
      "\"code\":\"OK\"," 
      "\"reason\":\"workspace_query_result\"," 
      "\"command_id\":\"yai.container.query\"," 
      "\"target_plane\":\"runtime\"," 
      "\"data\":{" 
      "\"binding_status\":\"active\"," 
      "\"workspace_id\":\"ws_model\"," 
      "\"query_family\":\"enforcement\"," 
      "\"execution_mode_degraded\":false"
      "}"
      "}";

  static const char *gov_json =
      "{"
      "\"type\":\"yai.exec.reply.v1\"," 
      "\"status\":\"error\"," 
      "\"code\":\"POLICY_BLOCK\"," 
      "\"reason\":\"deny\"," 
      "\"command_id\":\"yai.container.run\"," 
      "\"target_plane\":\"runtime\"," 
      "\"data\":{" 
      "\"decision\":{\"effect\":\"deny\"},"
      "\"enforcement\":{\"authority_decision\":\"deny\",\"review_state\":\"blocked\"},"
      "\"attachability\":{\"attachable\":false,\"blocked\":true}"
      "}"
      "}";

  static const char *op_json =
      "{"
      "\"type\":\"yai.exec.reply.v1\","
      "\"status\":\"ok\","
      "\"code\":\"OK\","
      "\"reason\":\"workspace_query_result\","
      "\"command_id\":\"yai.container.query\","
      "\"target_plane\":\"runtime\","
      "\"data\":{"
      "\"operational_summary\":{"
      "\"source_edge_summary\":{\"source_node\":3,\"source_daemon_instance\":2,\"source_binding\":4},"
      "\"delegation_summary\":{\"source_enrollment_grant\":2,\"source_policy_snapshot\":2,\"source_capability_envelope\":2},"
      "\"mesh_summary\":{\"mesh_coordination_membership\":3,\"mesh_peer_awareness\":4,\"mesh_peer_legitimacy\":3,\"mesh_authority_scope\":3},"
      "\"transport_ingress_overlay_summary\":{\"mesh_transport_endpoint\":2,\"mesh_transport_path_state\":2,\"mesh_owner_remote_ingress\":1,\"mesh_owner_remote_ingress_decision\":1,\"mesh_overlay_presence\":2,\"mesh_overlay_target_association\":2,\"mesh_overlay_path_mutation\":1}"
      "}"
      "}"
      "}";

  yai_sdk_runtime_state_t rs;
  yai_sdk_governance_state_t gs;
  yai_sdk_runtime_target_t target;
  yai_sdk_policy_distribution_descriptor_t dist;
  yai_sdk_remote_association_state_t remote;
  yai_sdk_operational_summary_t op;

  if (yai_sdk_control_call_to_json(&call, call_json, sizeof(call_json)) != 0) {
    fprintf(stderr, "models_contract_smoke: call json encode failed\n");
    return 1;
  }
  if (strstr(call_json, "yai.container.query") == NULL) {
    fprintf(stderr, "models_contract_smoke: call json missing command id\n");
    return 1;
  }

  if (yai_sdk_runtime_state_from_reply_json(reply_json, &rs) != 0) {
    fprintf(stderr, "models_contract_smoke: runtime state parse failed\n");
    return 2;
  }
  if (rs.liveness != YAI_SDK_LIVENESS_UP || rs.workspace_binding != YAI_SDK_BINDING_ACTIVE) {
    fprintf(stderr, "models_contract_smoke: runtime baseline/binding mismatch\n");
    return 2;
  }
  if (strcmp(rs.workspace_id, "ws_model") != 0) {
    fprintf(stderr, "models_contract_smoke: container id mismatch\n");
    return 2;
  }
  if (rs.data.ready != 1 || rs.data.bound != 1) {
    fprintf(stderr, "models_contract_smoke: data family readiness mismatch\n");
    return 2;
  }

  if (yai_sdk_governance_state_from_reply_json(gov_json, &gs) != 0) {
    fprintf(stderr, "models_contract_smoke: governance parse failed\n");
    return 3;
  }
  if (strcmp(gs.effect, "deny") != 0 || strcmp(gs.review_state, "blocked") != 0 || gs.blocked != 1) {
    fprintf(stderr, "models_contract_smoke: governance state mismatch\n");
    return 3;
  }

  yai_sdk_runtime_target_init(&target);
  yai_sdk_policy_distribution_descriptor_init(&dist);
  yai_sdk_remote_association_state_init(&remote);
  if (target.target_kind != YAI_SDK_RUNTIME_TARGET_UNKNOWN || target.mesh_plane != YAI_SDK_MESH_PLANE_UNKNOWN) {
    fprintf(stderr, "models_contract_smoke: target init mismatch\n");
    return 4;
  }
  if (dist.source_enrollment_grant_id[0] != '\0' || remote.reachability_state[0] != '\0') {
    fprintf(stderr, "models_contract_smoke: distribution/remote init mismatch\n");
    return 4;
  }

  if (yai_sdk_operational_summary_from_reply_json(op_json, &op) != 0) {
    fprintf(stderr, "models_contract_smoke: operational summary parse failed\n");
    return 5;
  }
  if (op.source_node_count != 3 || op.mesh_peer_legitimacy_count != 3 || op.mesh_overlay_path_mutation_count != 1) {
    fprintf(stderr, "models_contract_smoke: operational summary mismatch\n");
    return 5;
  }

  puts("models_contract_smoke: ok");
  return 0;
}
