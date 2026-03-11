/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/data/query.h>
#include <yai/daemon/source_plane_model.h>

#include <stdio.h>

#include "../store/internal.h"

typedef struct yai_data_operational_counts {
  size_t events;
  size_t decisions;
  size_t evidence;
  size_t governance;
  size_t authority;
  size_t authority_resolution;
  size_t enforcement_outcome;
  size_t enforcement_linkage;

  size_t source_node;
  size_t source_daemon_instance;
  size_t source_binding;
  size_t source_asset;
  size_t source_acquisition_event;
  size_t source_evidence_candidate;
  size_t source_ingest_outcome;
  size_t source_owner_link;
  size_t source_enrollment_grant;
  size_t source_policy_snapshot;
  size_t source_capability_envelope;
  size_t workspace_peer_membership;

  size_t mesh_node;
  size_t mesh_discovery_advertisement;
  size_t mesh_bootstrap_descriptor;
  size_t mesh_coordination_membership;
  size_t mesh_peer_awareness;
  size_t mesh_peer_legitimacy;
  size_t mesh_authority_scope;

  size_t mesh_transport_endpoint;
  size_t mesh_transport_path_state;
  size_t mesh_transport_channel_state;
  size_t mesh_owner_remote_ingress;
  size_t mesh_owner_remote_ingress_session;
  size_t mesh_owner_remote_ingress_decision;
  size_t mesh_overlay_presence;
  size_t mesh_overlay_target_association;
  size_t mesh_overlay_path_mutation;
} yai_data_operational_counts_t;

int yai_data_query_count(const char *workspace_id,
                         const char *record_class,
                         size_t *out_count,
                         char *err,
                         size_t err_cap)
{
  char file[768];
  if (err && err_cap > 0) err[0] = '\0';
  if (out_count) *out_count = 0;
  if (!workspace_id || !workspace_id[0] || !record_class || !record_class[0] || !out_count) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "workspace_or_class_missing");
    return -1;
  }
  {
    int duck_rc = yai_data_duckdb_count(workspace_id, record_class, out_count, err, err_cap);
    if (duck_rc == 0) return 0;
  }
  if (yai_data_store_paths(workspace_id, record_class, NULL, 0, file, sizeof(file), err, err_cap) != 0) return -1;
  return yai_data_store_count_lines(file, out_count, err, err_cap);
}

static int yai_data_query_collect_operational_counts(const char *workspace_id,
                                                     yai_data_operational_counts_t *counts,
                                                     char *err,
                                                     size_t err_cap)
{
  if (!workspace_id || !workspace_id[0] || !counts)
  {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "operational_summary_bad_args");
    return -1;
  }

  if (yai_data_query_count(workspace_id, "events", &counts->events, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, "decisions", &counts->decisions, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, "evidence", &counts->evidence, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, "governance", &counts->governance, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, "authority", &counts->authority, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, "authority_resolution", &counts->authority_resolution, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, "enforcement_outcome", &counts->enforcement_outcome, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, "enforcement_linkage", &counts->enforcement_linkage, err, err_cap) != 0) return -1;

  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_NODE, &counts->source_node, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_DAEMON_INSTANCE, &counts->source_daemon_instance, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_BINDING, &counts->source_binding, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_ASSET, &counts->source_asset, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_ACQUISITION_EVENT, &counts->source_acquisition_event, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_EVIDENCE_CANDIDATE, &counts->source_evidence_candidate, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_INGEST_OUTCOME, &counts->source_ingest_outcome, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_OWNER_LINK, &counts->source_owner_link, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_ENROLLMENT_GRANT, &counts->source_enrollment_grant, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_POLICY_SNAPSHOT, &counts->source_policy_snapshot, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_CAPABILITY_ENVELOPE, &counts->source_capability_envelope, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_WORKSPACE_PEER_MEMBERSHIP, &counts->workspace_peer_membership, err, err_cap) != 0) return -1;

  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_MESH_NODE, &counts->mesh_node, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_MESH_DISCOVERY_ADVERTISEMENT, &counts->mesh_discovery_advertisement, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_MESH_BOOTSTRAP_DESCRIPTOR, &counts->mesh_bootstrap_descriptor, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_MESH_COORDINATION_MEMBERSHIP, &counts->mesh_coordination_membership, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_MESH_PEER_AWARENESS, &counts->mesh_peer_awareness, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_MESH_PEER_LEGITIMACY, &counts->mesh_peer_legitimacy, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_MESH_AUTHORITY_SCOPE, &counts->mesh_authority_scope, err, err_cap) != 0) return -1;

  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_MESH_TRANSPORT_ENDPOINT, &counts->mesh_transport_endpoint, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_MESH_TRANSPORT_PATH_STATE, &counts->mesh_transport_path_state, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_MESH_TRANSPORT_CHANNEL_STATE, &counts->mesh_transport_channel_state, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_MESH_OWNER_REMOTE_INGRESS, &counts->mesh_owner_remote_ingress, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_MESH_OWNER_REMOTE_INGRESS_SESSION, &counts->mesh_owner_remote_ingress_session, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_MESH_OWNER_REMOTE_INGRESS_DECISION, &counts->mesh_owner_remote_ingress_decision, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_MESH_OVERLAY_PRESENCE, &counts->mesh_overlay_presence, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_MESH_OVERLAY_TARGET_ASSOCIATION, &counts->mesh_overlay_target_association, err, err_cap) != 0) return -1;
  if (yai_data_query_count(workspace_id, YAI_SOURCE_RECORD_CLASS_MESH_OVERLAY_PATH_MUTATION, &counts->mesh_overlay_path_mutation, err, err_cap) != 0) return -1;

  return 0;
}

int yai_data_query_summary_json(const char *workspace_id,
                                char *out_json,
                                size_t out_cap,
                                char *err,
                                size_t err_cap)
{
  yai_data_operational_counts_t counts = {0};

  if (!out_json || out_cap == 0) return -1;
  out_json[0] = '\0';

  if (yai_data_query_collect_operational_counts(workspace_id, &counts, err, err_cap) != 0) return -1;

  if (snprintf(out_json,
               out_cap,
               "{\"workspace_id\":\"%s\",\"counts\":{"
               "\"events\":%zu,\"decisions\":%zu,\"evidence\":%zu,\"governance\":%zu,\"authority\":%zu,\"authority_resolution\":%zu,\"enforcement_outcome\":%zu,\"enforcement_linkage\":%zu,"
               "\"source_node\":%zu,\"source_daemon_instance\":%zu,\"source_binding\":%zu,\"source_asset\":%zu,\"source_acquisition_event\":%zu,\"source_evidence_candidate\":%zu,\"source_ingest_outcome\":%zu,\"source_owner_link\":%zu,\"source_enrollment_grant\":%zu,\"source_policy_snapshot\":%zu,\"source_capability_envelope\":%zu,\"workspace_peer_membership\":%zu,"
               "\"mesh_node\":%zu,\"mesh_discovery_advertisement\":%zu,\"mesh_bootstrap_descriptor\":%zu,\"mesh_coordination_membership\":%zu,\"mesh_peer_awareness\":%zu,\"mesh_peer_legitimacy\":%zu,\"mesh_authority_scope\":%zu,"
               "\"mesh_transport_endpoint\":%zu,\"mesh_transport_path_state\":%zu,\"mesh_transport_channel_state\":%zu,\"mesh_owner_remote_ingress\":%zu,\"mesh_owner_remote_ingress_session\":%zu,\"mesh_owner_remote_ingress_decision\":%zu,\"mesh_overlay_presence\":%zu,\"mesh_overlay_target_association\":%zu,\"mesh_overlay_path_mutation\":%zu"
               "}}",
               workspace_id,
               counts.events,
               counts.decisions,
               counts.evidence,
               counts.governance,
               counts.authority,
               counts.authority_resolution,
               counts.enforcement_outcome,
               counts.enforcement_linkage,
               counts.source_node,
               counts.source_daemon_instance,
               counts.source_binding,
               counts.source_asset,
               counts.source_acquisition_event,
               counts.source_evidence_candidate,
               counts.source_ingest_outcome,
               counts.source_owner_link,
               counts.source_enrollment_grant,
               counts.source_policy_snapshot,
               counts.source_capability_envelope,
               counts.workspace_peer_membership,
               counts.mesh_node,
               counts.mesh_discovery_advertisement,
               counts.mesh_bootstrap_descriptor,
               counts.mesh_coordination_membership,
               counts.mesh_peer_awareness,
               counts.mesh_peer_legitimacy,
               counts.mesh_authority_scope,
               counts.mesh_transport_endpoint,
               counts.mesh_transport_path_state,
               counts.mesh_transport_channel_state,
               counts.mesh_owner_remote_ingress,
               counts.mesh_owner_remote_ingress_session,
               counts.mesh_owner_remote_ingress_decision,
               counts.mesh_overlay_presence,
               counts.mesh_overlay_target_association,
               counts.mesh_overlay_path_mutation) <= 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "summary_encode_failed");
    return -1;
  }
  return 0;
}

int yai_data_query_operational_summary_json(const char *workspace_id,
                                            char *out_json,
                                            size_t out_cap,
                                            char *err,
                                            size_t err_cap)
{
  yai_data_operational_counts_t counts = {0};

  if (!out_json || out_cap == 0) return -1;
  out_json[0] = '\0';

  if (yai_data_query_collect_operational_counts(workspace_id, &counts, err, err_cap) != 0) return -1;

  if (snprintf(out_json,
               out_cap,
               "{\"workspace_id\":\"%s\","
               "\"source_edge_summary\":{\"source_node\":%zu,\"source_daemon_instance\":%zu,\"source_binding\":%zu,\"source_asset\":%zu,\"source_acquisition_event\":%zu,\"source_evidence_candidate\":%zu,\"source_ingest_outcome\":%zu,\"workspace_peer_membership\":%zu},"
               "\"delegation_summary\":{\"source_enrollment_grant\":%zu,\"source_policy_snapshot\":%zu,\"source_capability_envelope\":%zu},"
               "\"mesh_summary\":{\"mesh_node\":%zu,\"mesh_discovery_advertisement\":%zu,\"mesh_bootstrap_descriptor\":%zu,\"mesh_coordination_membership\":%zu,\"mesh_peer_awareness\":%zu,\"mesh_peer_legitimacy\":%zu,\"mesh_authority_scope\":%zu},"
               "\"transport_ingress_overlay_summary\":{\"mesh_transport_endpoint\":%zu,\"mesh_transport_path_state\":%zu,\"mesh_transport_channel_state\":%zu,\"mesh_owner_remote_ingress\":%zu,\"mesh_owner_remote_ingress_session\":%zu,\"mesh_owner_remote_ingress_decision\":%zu,\"mesh_overlay_presence\":%zu,\"mesh_overlay_target_association\":%zu,\"mesh_overlay_path_mutation\":%zu},"
               "\"governance_signal_summary\":{\"events\":%zu,\"decisions\":%zu,\"evidence\":%zu,\"governance\":%zu,\"authority\":%zu,\"authority_resolution\":%zu,\"enforcement_outcome\":%zu,\"enforcement_linkage\":%zu}}",
               workspace_id,
               counts.source_node,
               counts.source_daemon_instance,
               counts.source_binding,
               counts.source_asset,
               counts.source_acquisition_event,
               counts.source_evidence_candidate,
               counts.source_ingest_outcome,
               counts.workspace_peer_membership,
               counts.source_enrollment_grant,
               counts.source_policy_snapshot,
               counts.source_capability_envelope,
               counts.mesh_node,
               counts.mesh_discovery_advertisement,
               counts.mesh_bootstrap_descriptor,
               counts.mesh_coordination_membership,
               counts.mesh_peer_awareness,
               counts.mesh_peer_legitimacy,
               counts.mesh_authority_scope,
               counts.mesh_transport_endpoint,
               counts.mesh_transport_path_state,
               counts.mesh_transport_channel_state,
               counts.mesh_owner_remote_ingress,
               counts.mesh_owner_remote_ingress_session,
               counts.mesh_owner_remote_ingress_decision,
               counts.mesh_overlay_presence,
               counts.mesh_overlay_target_association,
               counts.mesh_overlay_path_mutation,
               counts.events,
               counts.decisions,
               counts.evidence,
               counts.governance,
               counts.authority,
               counts.authority_resolution,
               counts.enforcement_outcome,
               counts.enforcement_linkage) <= 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "operational_summary_encode_failed");
    return -1;
  }

  return 0;
}

int yai_data_query_tail_json(const char *workspace_id,
                             const char *record_class,
                             size_t limit,
                             char *out_json,
                             size_t out_cap,
                             char *err,
                             size_t err_cap)
{
  return yai_data_store_tail_json(workspace_id, record_class, limit, out_json, out_cap, err, err_cap);
}
