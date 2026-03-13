int yai_session_build_workspace_policy_effective_json(char *out, size_t out_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];
    char sci_experiment[192];
    char sci_parameter[192];
    char sci_repro[192];
    char sci_dataset[192];
    char sci_publication[192];
    char dig_outbound[192];
    char dig_sink[192];
    char dig_publication[192];
    char dig_retrieval[192];
    char dig_distribution[192];
    char evt_declared[96];
    char evt_business[96];
    char evt_enforcement[96];
    char evt_stage[48];
    char evt_id[224];
    char sink_last_event_ref[96];
    char sink_last_decision_ref[96];
    char sink_last_evidence_ref[96];
    char sink_event_store_ref[MAX_PATH_LEN];
    char sink_decision_store_ref[MAX_PATH_LEN];
    char sink_evidence_store_ref[MAX_PATH_LEN];
    char enforce_last_outcome_ref[128];
    char enforce_last_linkage_ref[128];
    char enforce_materialization_status[48];
    char enforce_missing_fields[192];
    char enforce_outcome_store_ref[MAX_PATH_LEN];
    char enforce_linkage_store_ref[MAX_PATH_LEN];
    char gov_last_object_ref[256];
    char gov_last_lifecycle_ref[256];
    char gov_last_attachment_ref[320];
    char gov_object_store_ref[MAX_PATH_LEN];
    char gov_lifecycle_store_ref[MAX_PATH_LEN];
    char gov_attachment_store_ref[MAX_PATH_LEN];
    char authority_last_ref[192];
    char authority_resolution_ref[192];
    char artifact_last_ref[192];
    char artifact_linkage_ref[192];
    char authority_store_ref[MAX_PATH_LEN];
    char artifact_store_ref[MAX_PATH_LEN];
    char brain_graph_node_ref[192];
    char brain_graph_edge_ref[192];
    char brain_transient_state_ref[192];
    char brain_transient_working_set_ref[192];
    char brain_graph_store_ref[MAX_PATH_LEN];
    char brain_transient_store_ref[MAX_PATH_LEN];
    char read_primary_source[96];
    char read_fallback_reason[256];
    char runtime_caps[768];
    char op_summary[192];
    const char *review_state;
    int evt_external;
    int db_first_ready;
    int fallback_active;
    int n;
    if (!out || out_cap == 0)
        return -1;
    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err)) != 0)
    {
        n = snprintf(out, out_cap,
                     "{\"type\":\"yai.workspace.policy.effective.v1\",\"binding_status\":\"%s\",\"reason\":\"%s\"}",
                     status[0] ? status : "invalid",
                     err[0] ? err : "binding_error");
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }
    yai_workspace_build_scientific_summaries(&info,
                                             sci_experiment,
                                             sizeof(sci_experiment),
                                             sci_parameter,
                                             sizeof(sci_parameter),
                                             sci_repro,
                                             sizeof(sci_repro),
                                             sci_dataset,
                                             sizeof(sci_dataset),
                                             sci_publication,
                                             sizeof(sci_publication));
    yai_workspace_build_digital_summaries(&info,
                                          dig_outbound,
                                          sizeof(dig_outbound),
                                          dig_sink,
                                          sizeof(dig_sink),
                                          dig_publication,
                                          sizeof(dig_publication),
                                          dig_retrieval,
                                          sizeof(dig_retrieval),
                                          dig_distribution,
                                          sizeof(dig_distribution));
    yai_session_workspace_event_semantics(&info,
                                          evt_declared,
                                          sizeof(evt_declared),
                                          evt_business,
                                          sizeof(evt_business),
                                          evt_enforcement,
                                          sizeof(evt_enforcement),
                                          evt_stage,
                                          sizeof(evt_stage),
                                          &evt_external);
    snprintf(evt_id, sizeof(evt_id), "%s%s",
             info.last_resolution_trace_ref[0] ? "evt-" : "none",
             info.last_resolution_trace_ref[0] ? info.last_resolution_trace_ref : "");
    yai_workspace_read_event_evidence_index(&info,
                                            sink_last_event_ref,
                                            sizeof(sink_last_event_ref),
                                            sink_last_decision_ref,
                                            sizeof(sink_last_decision_ref),
                                            sink_last_evidence_ref,
                                            sizeof(sink_last_evidence_ref),
                                            sink_event_store_ref,
                                            sizeof(sink_event_store_ref),
                                            sink_decision_store_ref,
                                            sizeof(sink_decision_store_ref),
                                            sink_evidence_store_ref,
                                            sizeof(sink_evidence_store_ref));
    yai_workspace_read_enforcement_index(&info,
                                         enforce_last_outcome_ref,
                                         sizeof(enforce_last_outcome_ref),
                                         enforce_last_linkage_ref,
                                         sizeof(enforce_last_linkage_ref),
                                         enforce_materialization_status,
                                         sizeof(enforce_materialization_status),
                                         enforce_missing_fields,
                                         sizeof(enforce_missing_fields),
                                         enforce_outcome_store_ref,
                                         sizeof(enforce_outcome_store_ref),
                                         enforce_linkage_store_ref,
                                         sizeof(enforce_linkage_store_ref));
    yai_workspace_read_governance_index(&info,
                                        gov_last_object_ref,
                                        sizeof(gov_last_object_ref),
                                        gov_last_lifecycle_ref,
                                        sizeof(gov_last_lifecycle_ref),
                                        gov_last_attachment_ref,
                                        sizeof(gov_last_attachment_ref),
                                        gov_object_store_ref,
                                        sizeof(gov_object_store_ref),
                                        gov_lifecycle_store_ref,
                                        sizeof(gov_lifecycle_store_ref),
                                        gov_attachment_store_ref,
                                        sizeof(gov_attachment_store_ref));
    yai_workspace_read_authority_artifact_indexes(&info,
                                                  authority_last_ref,
                                                  sizeof(authority_last_ref),
                                                  authority_resolution_ref,
                                                  sizeof(authority_resolution_ref),
                                                  artifact_last_ref,
                                                  sizeof(artifact_last_ref),
                                                  artifact_linkage_ref,
                                                  sizeof(artifact_linkage_ref),
                                                  authority_store_ref,
                                                  sizeof(authority_store_ref),
                                                  artifact_store_ref,
                                                  sizeof(artifact_store_ref));
    (void)yai_storage_bridge_last_refs(info.ws_id,
                                            brain_graph_node_ref,
                                            sizeof(brain_graph_node_ref),
                                            brain_graph_edge_ref,
                                            sizeof(brain_graph_edge_ref),
                                            brain_transient_state_ref,
                                            sizeof(brain_transient_state_ref),
                                            brain_transient_working_set_ref,
                                            sizeof(brain_transient_working_set_ref),
                                            brain_graph_store_ref,
                                            sizeof(brain_graph_store_ref),
                                            brain_transient_store_ref,
                                            sizeof(brain_transient_store_ref));
    yai_workspace_db_first_read_model("policy_effective",
                                      sink_last_event_ref,
                                      sink_last_decision_ref,
                                      sink_last_evidence_ref,
                                      gov_last_object_ref,
                                      gov_last_lifecycle_ref,
                                      authority_last_ref,
                                      artifact_last_ref,
                                      brain_graph_node_ref,
                                      brain_graph_edge_ref,
                                      enforce_last_outcome_ref,
                                      enforce_last_linkage_ref,
                                      enforce_materialization_status,
                                      read_primary_source,
                                      sizeof(read_primary_source),
                                      read_fallback_reason,
                                      sizeof(read_fallback_reason),
                                      &db_first_ready,
                                      &fallback_active);
    review_state = yai_workspace_review_state_from_effect(info.last_effect_summary);
    yai_workspace_operational_summary(evt_stage, evt_business, info.last_effect_summary, op_summary, sizeof(op_summary));
    if (yai_workspace_build_runtime_capabilities_json(&info,
                                                      "active",
                                                      runtime_caps,
                                                      sizeof(runtime_caps)) != 0)
    {
        snprintf(runtime_caps, sizeof(runtime_caps), "%s", "{}");
    }
    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.policy.effective.v1\","
                 "\"workspace_id\":\"%s\","
                 "\"workspace_namespace\":\"%s\","
                 "\"namespace_valid\":%s,"
                 "\"containment_ready\":%s,"
                 "\"security_level_effective\":\"%s\","
                 "\"execution_mode_requested\":\"%s\","
                 "\"execution_mode_effective\":\"%s\","
                 "\"execution_mode_degraded\":%s,"
                 "\"execution_degraded_reason\":\"%s\","
                 "\"execution_unsupported_scopes\":\"%s\","
                 "\"execution_profile_ref\":\"%s\","
                 "\"traces_index_ref\":\"%s\","
                 "\"artifacts_index_ref\":\"%s\","
                 "\"family_effective\":\"%s\","
                 "\"specialization_effective\":\"%s\","
                 "\"event_surface\":{\"event_id\":\"%s\",\"flow_stage\":\"%s\",\"declared_scenario_specialization\":\"%s\",\"business_specialization\":\"%s\",\"enforcement_specialization\":\"%s\",\"external_effect_boundary\":%s},"
                 "\"operational_state\":{\"binding_state\":\"active\",\"attached_governance_objects\":\"%s\",\"active_effective_stack\":\"%s\",\"last_event_ref\":\"%s\",\"last_flow_stage\":\"%s\",\"last_business_specialization\":\"%s\",\"last_enforcement_specialization\":\"%s\",\"last_effect\":\"%s\",\"last_authority\":\"%s\",\"last_evidence\":\"%s\",\"last_trace_ref\":\"%s\",\"review_state\":\"%s\",\"operational_summary\":\"%s\"},"
                 "\"effective_stack_ref\":\"%s\","
                 "\"effective_overlays_ref\":\"%s\","
                 "\"policy_attachments\":\"%s\","
                 "\"policy_attachment_count\":%d,"
                 "\"precedence\":\"specialization+overlays\","
                 "\"effect_summary\":\"%s\","
                 "\"authority_summary\":\"%s\","
                 "\"evidence_summary\":\"%s\","
                 "\"event_evidence_sink\":{\"last_event_ref\":\"%s\",\"last_decision_ref\":\"%s\",\"last_evidence_ref\":\"%s\",\"event_store_ref\":\"%s\",\"decision_store_ref\":\"%s\",\"evidence_store_ref\":\"%s\"},"
                 "\"governance_persistence\":{\"last_object_ref\":\"%s\",\"last_lifecycle_ref\":\"%s\",\"last_attachment_ref\":\"%s\",\"object_store_ref\":\"%s\",\"lifecycle_store_ref\":\"%s\",\"attachment_store_ref\":\"%s\"},"
                 "\"authority_artifact_persistence\":{\"last_authority_ref\":\"%s\",\"last_authority_resolution_ref\":\"%s\",\"last_artifact_ref\":\"%s\",\"last_artifact_linkage_ref\":\"%s\",\"authority_store_ref\":\"%s\",\"artifact_store_ref\":\"%s\"},"
                 "\"graph_persistence\":{\"last_graph_node_ref\":\"%s\",\"last_graph_edge_ref\":\"%s\",\"graph_store_ref\":\"%s\",\"graph_truth_authoritative\":true},"
                 "\"knowledge_transient_persistence\":{\"last_transient_state_ref\":\"%s\",\"last_transient_working_set_ref\":\"%s\",\"transient_store_ref\":\"%s\",\"transient_authoritative\":false},"
                 "\"runtime_capabilities\":%s,"
                 "\"enforcement_record_set\":{\"last_outcome_ref\":\"%s\",\"last_linkage_ref\":\"%s\",\"materialization_status\":\"%s\",\"missing_fields\":\"%s\",\"outcome_store_ref\":\"%s\",\"linkage_store_ref\":\"%s\"},"
                 "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"%s\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                 "\"scientific\":{\"experiment_context_summary\":\"%s\",\"parameter_governance_summary\":\"%s\",\"reproducibility_summary\":\"%s\",\"dataset_integrity_summary\":\"%s\",\"publication_control_summary\":\"%s\"},"
                 "\"digital\":{\"outbound_context_summary\":\"%s\",\"sink_target_summary\":\"%s\",\"publication_control_summary\":\"%s\",\"retrieval_control_summary\":\"%s\",\"distribution_control_summary\":\"%s\"}"
                 "}",
                 info.ws_id,
                 info.workspace_namespace,
                 info.namespace_valid ? "true" : "false",
                 info.containment_ready ? "true" : "false",
                 info.security_level_effective[0] ? info.security_level_effective : "logical",
                 info.execution_mode_requested[0] ? info.execution_mode_requested : "scoped",
                 info.execution_mode_effective[0] ? info.execution_mode_effective : "scoped",
                 info.execution_mode_degraded ? "true" : "false",
                 info.execution_degraded_reason[0] ? info.execution_degraded_reason : "none",
                 info.execution_unsupported_scopes[0] ? info.execution_unsupported_scopes : "none",
                 info.execution_profile_ref,
                 info.traces_index_path,
                 info.artifacts_index_path,
                 info.inferred_family[0] ? info.inferred_family : info.declared_control_family,
                 info.inferred_specialization[0] ? info.inferred_specialization : info.declared_specialization,
                 evt_id,
                 evt_stage,
                 evt_declared,
                 evt_business,
                 evt_enforcement,
                 evt_external ? "true" : "false",
                 info.policy_attachments_csv,
                 info.effective_stack_ref,
                 evt_id,
                 evt_stage,
                 evt_business,
                 evt_enforcement,
                 info.last_effect_summary,
                 info.last_authority_summary,
                 info.last_evidence_summary,
                 info.last_resolution_trace_ref,
                 review_state,
                 op_summary,
                 info.effective_stack_ref,
                 info.effective_overlays_ref,
                 info.policy_attachments_csv,
                 info.policy_attachment_count,
                 info.last_effect_summary,
                 info.last_authority_summary,
                 info.last_evidence_summary,
                 sink_last_event_ref[0] ? sink_last_event_ref : evt_id,
                 sink_last_decision_ref,
                 sink_last_evidence_ref,
                 sink_event_store_ref,
                 sink_decision_store_ref,
                 sink_evidence_store_ref,
                 gov_last_object_ref,
                 gov_last_lifecycle_ref,
                 gov_last_attachment_ref,
                 gov_object_store_ref,
                 gov_lifecycle_store_ref,
                 gov_attachment_store_ref,
                 authority_last_ref,
                 authority_resolution_ref,
                 artifact_last_ref,
                 artifact_linkage_ref,
                 authority_store_ref,
                 artifact_store_ref,
                 brain_graph_node_ref,
                 brain_graph_edge_ref,
                 brain_graph_store_ref,
                 brain_transient_state_ref,
                 brain_transient_working_set_ref,
                 brain_transient_store_ref,
                 runtime_caps,
                 enforce_last_outcome_ref,
                 enforce_last_linkage_ref,
                 enforce_materialization_status[0] ? enforce_materialization_status : "unknown",
                 enforce_missing_fields,
                 enforce_outcome_store_ref,
                 enforce_linkage_store_ref,
                 read_primary_source,
                 db_first_ready ? "true" : "false",
                 fallback_active ? "true" : "false",
                 read_fallback_reason,
                 sci_experiment,
                 sci_parameter,
                 sci_repro,
                 sci_dataset,
                 sci_publication,
                 dig_outbound,
                 dig_sink,
                 dig_publication,
                 dig_retrieval,
                 dig_distribution);
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}

int yai_session_build_workspace_debug_resolution_json(char *out, size_t out_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char err[96];
    char sci_experiment[192];
    char sci_parameter[192];
    char sci_repro[192];
    char sci_dataset[192];
    char sci_publication[192];
    char dig_outbound[192];
    char dig_sink[192];
    char dig_publication[192];
    char dig_retrieval[192];
    char dig_distribution[192];
    char evt_declared[96];
    char evt_business[96];
    char evt_enforcement[96];
    char evt_stage[48];
    char evt_id[224];
    char sink_last_event_ref[96];
    char sink_last_decision_ref[96];
    char sink_last_evidence_ref[96];
    char sink_event_store_ref[MAX_PATH_LEN];
    char sink_decision_store_ref[MAX_PATH_LEN];
    char sink_evidence_store_ref[MAX_PATH_LEN];
    char enforce_last_outcome_ref[128];
    char enforce_last_linkage_ref[128];
    char enforce_materialization_status[48];
    char enforce_missing_fields[192];
    char enforce_outcome_store_ref[MAX_PATH_LEN];
    char enforce_linkage_store_ref[MAX_PATH_LEN];
    char gov_last_object_ref[256];
    char gov_last_lifecycle_ref[256];
    char gov_last_attachment_ref[320];
    char gov_object_store_ref[MAX_PATH_LEN];
    char gov_lifecycle_store_ref[MAX_PATH_LEN];
    char gov_attachment_store_ref[MAX_PATH_LEN];
    char authority_last_ref[192];
    char authority_resolution_ref[192];
    char artifact_last_ref[192];
    char artifact_linkage_ref[192];
    char authority_store_ref[MAX_PATH_LEN];
    char artifact_store_ref[MAX_PATH_LEN];
    char brain_graph_node_ref[192];
    char brain_graph_edge_ref[192];
    char brain_transient_state_ref[192];
    char brain_transient_working_set_ref[192];
    char brain_graph_store_ref[MAX_PATH_LEN];
    char brain_transient_store_ref[MAX_PATH_LEN];
    char read_primary_source[96];
    char read_fallback_reason[256];
    char runtime_caps[768];
    char op_summary[192];
    const char *review_state;
    int evt_external;
    int db_first_ready;
    int fallback_active;
    int n;
    const char *context_source;
    if (!out || out_cap == 0)
        return -1;
    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), err, sizeof(err)) != 0)
    {
        n = snprintf(out, out_cap,
                     "{\"type\":\"yai.workspace.debug.resolution.v1\",\"binding_status\":\"%s\",\"reason\":\"%s\"}",
                     status[0] ? status : "invalid",
                     err[0] ? err : "binding_error");
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }
    context_source = info.declared_context_source[0] ? info.declared_context_source : "unset";
    yai_workspace_build_scientific_summaries(&info,
                                             sci_experiment,
                                             sizeof(sci_experiment),
                                             sci_parameter,
                                             sizeof(sci_parameter),
                                             sci_repro,
                                             sizeof(sci_repro),
                                             sci_dataset,
                                             sizeof(sci_dataset),
                                             sci_publication,
                                             sizeof(sci_publication));
    yai_workspace_build_digital_summaries(&info,
                                          dig_outbound,
                                          sizeof(dig_outbound),
                                          dig_sink,
                                          sizeof(dig_sink),
                                          dig_publication,
                                          sizeof(dig_publication),
                                          dig_retrieval,
                                          sizeof(dig_retrieval),
                                          dig_distribution,
                                          sizeof(dig_distribution));
    yai_session_workspace_event_semantics(&info,
                                          evt_declared,
                                          sizeof(evt_declared),
                                          evt_business,
                                          sizeof(evt_business),
                                          evt_enforcement,
                                          sizeof(evt_enforcement),
                                          evt_stage,
                                          sizeof(evt_stage),
                                          &evt_external);
    snprintf(evt_id, sizeof(evt_id), "%s%s",
             info.last_resolution_trace_ref[0] ? "evt-" : "none",
             info.last_resolution_trace_ref[0] ? info.last_resolution_trace_ref : "");
    yai_workspace_read_event_evidence_index(&info,
                                            sink_last_event_ref,
                                            sizeof(sink_last_event_ref),
                                            sink_last_decision_ref,
                                            sizeof(sink_last_decision_ref),
                                            sink_last_evidence_ref,
                                            sizeof(sink_last_evidence_ref),
                                            sink_event_store_ref,
                                            sizeof(sink_event_store_ref),
                                            sink_decision_store_ref,
                                            sizeof(sink_decision_store_ref),
                                            sink_evidence_store_ref,
                                            sizeof(sink_evidence_store_ref));
    yai_workspace_read_enforcement_index(&info,
                                         enforce_last_outcome_ref,
                                         sizeof(enforce_last_outcome_ref),
                                         enforce_last_linkage_ref,
                                         sizeof(enforce_last_linkage_ref),
                                         enforce_materialization_status,
                                         sizeof(enforce_materialization_status),
                                         enforce_missing_fields,
                                         sizeof(enforce_missing_fields),
                                         enforce_outcome_store_ref,
                                         sizeof(enforce_outcome_store_ref),
                                         enforce_linkage_store_ref,
                                         sizeof(enforce_linkage_store_ref));
    yai_workspace_read_governance_index(&info,
                                        gov_last_object_ref,
                                        sizeof(gov_last_object_ref),
                                        gov_last_lifecycle_ref,
                                        sizeof(gov_last_lifecycle_ref),
                                        gov_last_attachment_ref,
                                        sizeof(gov_last_attachment_ref),
                                        gov_object_store_ref,
                                        sizeof(gov_object_store_ref),
                                        gov_lifecycle_store_ref,
                                        sizeof(gov_lifecycle_store_ref),
                                        gov_attachment_store_ref,
                                        sizeof(gov_attachment_store_ref));
    yai_workspace_read_authority_artifact_indexes(&info,
                                                  authority_last_ref,
                                                  sizeof(authority_last_ref),
                                                  authority_resolution_ref,
                                                  sizeof(authority_resolution_ref),
                                                  artifact_last_ref,
                                                  sizeof(artifact_last_ref),
                                                  artifact_linkage_ref,
                                                  sizeof(artifact_linkage_ref),
                                                  authority_store_ref,
                                                  sizeof(authority_store_ref),
                                                  artifact_store_ref,
                                                  sizeof(artifact_store_ref));
    (void)yai_storage_bridge_last_refs(info.ws_id,
                                            brain_graph_node_ref,
                                            sizeof(brain_graph_node_ref),
                                            brain_graph_edge_ref,
                                            sizeof(brain_graph_edge_ref),
                                            brain_transient_state_ref,
                                            sizeof(brain_transient_state_ref),
                                            brain_transient_working_set_ref,
                                            sizeof(brain_transient_working_set_ref),
                                            brain_graph_store_ref,
                                            sizeof(brain_graph_store_ref),
                                            brain_transient_store_ref,
                                            sizeof(brain_transient_store_ref));
    yai_workspace_db_first_read_model("debug_resolution",
                                      sink_last_event_ref,
                                      sink_last_decision_ref,
                                      sink_last_evidence_ref,
                                      gov_last_object_ref,
                                      gov_last_lifecycle_ref,
                                      authority_last_ref,
                                      artifact_last_ref,
                                      brain_graph_node_ref,
                                      brain_graph_edge_ref,
                                      enforce_last_outcome_ref,
                                      enforce_last_linkage_ref,
                                      enforce_materialization_status,
                                      read_primary_source,
                                      sizeof(read_primary_source),
                                      read_fallback_reason,
                                      sizeof(read_fallback_reason),
                                      &db_first_ready,
                                      &fallback_active);
    review_state = yai_workspace_review_state_from_effect(info.last_effect_summary);
    yai_workspace_operational_summary(evt_stage, evt_business, info.last_effect_summary, op_summary, sizeof(op_summary));
    if (yai_workspace_build_runtime_capabilities_json(&info,
                                                      "active",
                                                      runtime_caps,
                                                      sizeof(runtime_caps)) != 0)
    {
        snprintf(runtime_caps, sizeof(runtime_caps), "%s", "{}");
    }
    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.debug.resolution.v1\","
                 "\"workspace_id\":\"%s\","
                 "\"workspace_namespace\":\"%s\","
                 "\"namespace_valid\":%s,"
                 "\"containment_ready\":%s,"
                 "\"security_level_effective\":\"%s\","
                 "\"execution_mode_requested\":\"%s\","
                 "\"execution_mode_effective\":\"%s\","
                 "\"execution_mode_degraded\":%s,"
                 "\"execution_degraded_reason\":\"%s\","
                 "\"execution_unsupported_scopes\":\"%s\","
                 "\"attach_descriptor_ref\":\"%s\","
                 "\"state_surface_ref\":\"%s\","
                 "\"context_source\":\"%s\","
                 "\"declared\":{\"family\":\"%s\",\"specialization\":\"%s\"},"
                 "\"inferred\":{\"family\":\"%s\",\"specialization\":\"%s\",\"confidence\":%.3f},"
                 "\"event_surface\":{\"event_id\":\"%s\",\"flow_stage\":\"%s\",\"declared_scenario_specialization\":\"%s\",\"business_specialization\":\"%s\",\"enforcement_specialization\":\"%s\",\"external_effect_boundary\":%s},"
                 "\"operational_state\":{\"binding_state\":\"active\",\"attached_governance_objects\":\"%s\",\"active_effective_stack\":\"%s\",\"last_event_ref\":\"%s\",\"last_flow_stage\":\"%s\",\"last_business_specialization\":\"%s\",\"last_enforcement_specialization\":\"%s\",\"last_effect\":\"%s\",\"last_authority\":\"%s\",\"last_evidence\":\"%s\",\"last_trace_ref\":\"%s\",\"review_state\":\"%s\",\"operational_summary\":\"%s\"},"
                 "\"effective\":{\"stack_ref\":\"%s\",\"overlays_ref\":\"%s\"},"
                 "\"precedence_outcome\":\"%s\","
                 "\"effect_outcome\":\"%s\","
                 "\"last_resolution_trace_ref\":\"%s\","
                 "\"last_resolution_summary\":\"%s\","
                 "\"event_evidence_sink\":{\"last_event_ref\":\"%s\",\"last_decision_ref\":\"%s\",\"last_evidence_ref\":\"%s\",\"event_store_ref\":\"%s\",\"decision_store_ref\":\"%s\",\"evidence_store_ref\":\"%s\"},"
                 "\"governance_persistence\":{\"last_object_ref\":\"%s\",\"last_lifecycle_ref\":\"%s\",\"last_attachment_ref\":\"%s\",\"object_store_ref\":\"%s\",\"lifecycle_store_ref\":\"%s\",\"attachment_store_ref\":\"%s\"},"
                 "\"authority_artifact_persistence\":{\"last_authority_ref\":\"%s\",\"last_authority_resolution_ref\":\"%s\",\"last_artifact_ref\":\"%s\",\"last_artifact_linkage_ref\":\"%s\",\"authority_store_ref\":\"%s\",\"artifact_store_ref\":\"%s\"},"
                 "\"graph_persistence\":{\"last_graph_node_ref\":\"%s\",\"last_graph_edge_ref\":\"%s\",\"graph_store_ref\":\"%s\",\"graph_truth_authoritative\":true},"
                 "\"knowledge_transient_persistence\":{\"last_transient_state_ref\":\"%s\",\"last_transient_working_set_ref\":\"%s\",\"transient_store_ref\":\"%s\",\"transient_authoritative\":false},"
                 "\"runtime_capabilities\":%s,"
                 "\"enforcement_record_set\":{\"last_outcome_ref\":\"%s\",\"last_linkage_ref\":\"%s\",\"materialization_status\":\"%s\",\"missing_fields\":\"%s\",\"outcome_store_ref\":\"%s\",\"linkage_store_ref\":\"%s\"},"
                 "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"%s\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                 "\"scientific\":{\"experiment_context_summary\":\"%s\",\"parameter_governance_summary\":\"%s\",\"reproducibility_summary\":\"%s\",\"dataset_integrity_summary\":\"%s\",\"publication_control_summary\":\"%s\"},"
                 "\"digital\":{\"outbound_context_summary\":\"%s\",\"sink_target_summary\":\"%s\",\"publication_control_summary\":\"%s\",\"retrieval_control_summary\":\"%s\",\"distribution_control_summary\":\"%s\"}"
                 "}",
                 info.ws_id,
                 info.workspace_namespace,
                 info.namespace_valid ? "true" : "false",
                 info.containment_ready ? "true" : "false",
                 info.security_level_effective[0] ? info.security_level_effective : "logical",
                 info.execution_mode_requested[0] ? info.execution_mode_requested : "scoped",
                 info.execution_mode_effective[0] ? info.execution_mode_effective : "scoped",
                 info.execution_mode_degraded ? "true" : "false",
                 info.execution_degraded_reason[0] ? info.execution_degraded_reason : "none",
                 info.execution_unsupported_scopes[0] ? info.execution_unsupported_scopes : "none",
                 info.attach_descriptor_ref,
                 info.state_surface_path,
                 context_source,
                 info.declared_control_family,
                 info.declared_specialization,
                 info.inferred_family,
                 info.inferred_specialization,
                 info.inferred_confidence,
                 evt_id,
                 evt_stage,
                 evt_declared,
                 evt_business,
                 evt_enforcement,
                 evt_external ? "true" : "false",
                 info.policy_attachments_csv,
                 info.effective_stack_ref,
                 evt_id,
                 evt_stage,
                 evt_business,
                 evt_enforcement,
                 info.last_effect_summary,
                 info.last_authority_summary,
                 info.last_evidence_summary,
                 info.last_resolution_trace_ref,
                 review_state,
                 op_summary,
                 info.effective_stack_ref,
                 info.effective_overlays_ref,
                 "specialization+overlays",
                 info.last_effect_summary,
                 info.last_resolution_trace_ref,
                 info.last_resolution_summary,
                 sink_last_event_ref[0] ? sink_last_event_ref : evt_id,
                 sink_last_decision_ref,
                 sink_last_evidence_ref,
                 sink_event_store_ref,
                 sink_decision_store_ref,
                 sink_evidence_store_ref,
                 gov_last_object_ref,
                 gov_last_lifecycle_ref,
                 gov_last_attachment_ref,
                 gov_object_store_ref,
                 gov_lifecycle_store_ref,
                 gov_attachment_store_ref,
                 authority_last_ref,
                 authority_resolution_ref,
                 artifact_last_ref,
                 artifact_linkage_ref,
                 authority_store_ref,
                 artifact_store_ref,
                 brain_graph_node_ref,
                 brain_graph_edge_ref,
                 brain_graph_store_ref,
                 brain_transient_state_ref,
                 brain_transient_working_set_ref,
                 brain_transient_store_ref,
                 runtime_caps,
                 enforce_last_outcome_ref,
                 enforce_last_linkage_ref,
                 enforce_materialization_status[0] ? enforce_materialization_status : "unknown",
                 enforce_missing_fields,
                 enforce_outcome_store_ref,
                 enforce_linkage_store_ref,
                 read_primary_source,
                 db_first_ready ? "true" : "false",
                 fallback_active ? "true" : "false",
                 read_fallback_reason,
                 sci_experiment,
                 sci_parameter,
                 sci_repro,
                 sci_dataset,
                 sci_publication,
                 dig_outbound,
                 dig_sink,
                 dig_publication,
                 dig_retrieval,
                 dig_distribution);
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}

static int yai_workspace_graph_paths(const char *ws_id,
                                     char *nodes_log,
                                     size_t nodes_log_cap,
                                     char *edges_log,
                                     size_t edges_log_cap,
                                     char *index_path,
                                     size_t index_path_cap)
{
    char run_dir[MAX_PATH_LEN];
    char legacy_nodes[MAX_PATH_LEN];
    char legacy_edges[MAX_PATH_LEN];
    char legacy_index[MAX_PATH_LEN];
    if (!ws_id || !nodes_log || !edges_log || !index_path)
        return -1;
    if (yai_session_build_run_path(run_dir, sizeof(run_dir), ws_id) != 0)
        return -1;
    if (snprintf(nodes_log, nodes_log_cap, "%s/runtime/graph/persistent-nodes.v1.ndjson", run_dir) <= 0)
        return -1;
    if (snprintf(edges_log, edges_log_cap, "%s/runtime/graph/persistent-edges.v1.ndjson", run_dir) <= 0)
        return -1;
    if (snprintf(index_path, index_path_cap, "%s/runtime/graph/index.v1.json", run_dir) <= 0)
        return -1;
    if (!yai_session_path_exists(index_path))
    {
        if (snprintf(legacy_nodes, sizeof(legacy_nodes), "%s/brain/graph/persistent-nodes.v1.ndjson", run_dir) <= 0 ||
            snprintf(legacy_edges, sizeof(legacy_edges), "%s/brain/graph/persistent-edges.v1.ndjson", run_dir) <= 0 ||
            snprintf(legacy_index, sizeof(legacy_index), "%s/brain/graph/index.v1.json", run_dir) <= 0)
            return -1;
        if (yai_session_path_exists(legacy_index))
        {
            (void)snprintf(nodes_log, nodes_log_cap, "%s", legacy_nodes);
            (void)snprintf(edges_log, edges_log_cap, "%s", legacy_edges);
            (void)snprintf(index_path, index_path_cap, "%s", legacy_index);
        }
    }
    return 0;
}

static int yai_graph_rows_from_csv(const char *csv,
                                   const char *scope,
                                   char *rows_out,
                                   size_t rows_out_cap,
                                   int max_rows)
{
    char copy[1024];
    char *tok = NULL;
    char *save = NULL;
    size_t used = 0;
    int count = 0;
    int n;

    if (!rows_out || rows_out_cap == 0)
        return -1;
    rows_out[0] = '\0';
    if (!csv || !csv[0])
    {
        (void)snprintf(rows_out, rows_out_cap, "%s", "");
        return 0;
    }
    if (strlen(csv) >= sizeof(copy))
        return -1;
    (void)snprintf(copy, sizeof(copy), "%s", csv);
    tok = strtok_r(copy, ",", &save);
    while (tok && count < max_rows)
    {
        while (*tok == ' ')
            tok++;
        if (*tok)
        {
            n = snprintf(rows_out + used,
                         rows_out_cap - used,
                         "%s[\"%s\",\"%s\"]",
                         used ? "," : "",
                         tok,
                         scope && scope[0] ? scope : "relation");
            if (n <= 0 || (size_t)n >= (rows_out_cap - used))
                return -1;
            used += (size_t)n;
            count++;
        }
        tok = strtok_r(NULL, ",", &save);
    }
    return 0;
}

static int yai_authority_requirement_coverage_from_csv(const char *csv,
                                                       const char *authority_decision,
                                                       char *rows_out,
                                                       size_t rows_out_cap,
                                                       int max_rows,
                                                       int *out_all_covered)
{
    char copy[512];
    char *tok = NULL;
    char *save = NULL;
    size_t used = 0;
    int count = 0;
    int all_covered = 1;
    int n;
    int covered;

    if (!rows_out || rows_out_cap == 0)
        return -1;
    rows_out[0] = '\0';
    if (out_all_covered)
        *out_all_covered = 0;

    if (!csv || !csv[0])
        return 0;

    if (strlen(csv) >= sizeof(copy))
        return -1;

    (void)snprintf(copy, sizeof(copy), "%s", csv);
    tok = strtok_r(copy, ",", &save);
    while (tok && count < max_rows)
    {
        while (*tok == ' ')
            tok++;
        if (*tok)
        {
            covered = (authority_decision &&
                       authority_decision[0] &&
                       strcmp(authority_decision, "deny") != 0) ? 1 : 0;
            n = snprintf(rows_out + used,
                         rows_out_cap - used,
                         "%s{\"requirement\":\"%s\",\"covered\":%s}",
                         used ? "," : "",
                         tok,
                         covered ? "true" : "false");
            if (n <= 0 || (size_t)n >= (rows_out_cap - used))
                return -1;
            used += (size_t)n;
            if (!covered)
                all_covered = 0;
            count++;
        }
        tok = strtok_r(NULL, ",", &save);
    }

    if (count == 0)
        all_covered = 0;
    if (out_all_covered)
        *out_all_covered = all_covered;
    return 0;
}

static void yai_graph_load_index(const char *index_path,
                                 char *node_classes,
                                 size_t node_classes_cap,
                                 char *edge_classes,
                                 size_t edge_classes_cap,
                                 char *updated_at,
                                 size_t updated_at_cap,
                                 long *graph_node_count,
                                 long *graph_edge_count)
{
    char buf[8192];
    long n_nodes = 0;
    long n_edges = 0;
    if (node_classes && node_classes_cap > 0) node_classes[0] = '\0';
    if (edge_classes && edge_classes_cap > 0) edge_classes[0] = '\0';
    if (updated_at && updated_at_cap > 0) updated_at[0] = '\0';
    if (graph_node_count) *graph_node_count = 0;
    if (graph_edge_count) *graph_edge_count = 0;
    if (!index_path || yai_read_text(index_path, buf, sizeof(buf)) != 0)
        return;
    if (node_classes && node_classes_cap > 0)
        (void)yai_session_extract_json_string(buf, "node_classes", node_classes, node_classes_cap);
    if (edge_classes && edge_classes_cap > 0)
        (void)yai_session_extract_json_string(buf, "edge_classes", edge_classes, edge_classes_cap);
    if (updated_at && updated_at_cap > 0)
        (void)yai_session_extract_json_string(buf, "updated_at", updated_at, updated_at_cap);
    if (yai_session_extract_json_long(buf, "graph_node_count", &n_nodes) == 0 && graph_node_count)
        *graph_node_count = n_nodes;
    if (yai_session_extract_json_long(buf, "graph_edge_count", &n_edges) == 0 && graph_edge_count)
        *graph_edge_count = n_edges;
}

int yai_session_build_workspace_data_query_json(const char *query_family,
                                                char *out,
                                                size_t out_cap,
                                                char *err,
                                                size_t err_cap)
{
    yai_workspace_runtime_info_t info;
    char status[24];
    char bind_err[96];
    char qf[48];
    char sink_last_event_ref[96];
    char sink_last_decision_ref[96];
    char sink_last_evidence_ref[96];
    char sink_event_store_ref[MAX_PATH_LEN];
    char sink_decision_store_ref[MAX_PATH_LEN];
    char sink_evidence_store_ref[MAX_PATH_LEN];
    char enforce_last_outcome_ref[128];
    char enforce_last_linkage_ref[128];
    char enforce_materialization_status[48];
    char enforce_missing_fields[192];
    char enforce_outcome_store_ref[MAX_PATH_LEN];
    char enforce_linkage_store_ref[MAX_PATH_LEN];
    char gov_last_object_ref[256];
    char gov_last_lifecycle_ref[256];
    char gov_last_attachment_ref[320];
    char gov_object_store_ref[MAX_PATH_LEN];
    char gov_lifecycle_store_ref[MAX_PATH_LEN];
    char gov_attachment_store_ref[MAX_PATH_LEN];
    char authority_last_ref[192];
    char authority_resolution_ref[192];
    char artifact_last_ref[192];
    char artifact_linkage_ref[192];
    char authority_store_ref[MAX_PATH_LEN];
    char artifact_store_ref[MAX_PATH_LEN];
    char brain_graph_node_ref[192];
    char brain_graph_edge_ref[192];
    char brain_transient_state_ref[192];
    char brain_transient_working_set_ref[192];
    char brain_graph_store_ref[MAX_PATH_LEN];
    char brain_transient_store_ref[MAX_PATH_LEN];
    char graph_nodes_log[MAX_PATH_LEN];
    char graph_edges_log[MAX_PATH_LEN];
    char graph_index_path[MAX_PATH_LEN];
    char graph_node_classes[1024];
    char graph_edge_classes[1024];
    char graph_updated_at[64];
    char graph_node_rows_json[4096];
    char graph_edge_rows_json[4096];
    char graph_query_summary_json[1024];
    char graph_query_err[96];
    const char *qf_model;
    long graph_node_count;
    long graph_edge_count;
    char read_primary_source[96];
    char read_fallback_reason[256];
    char lifecycle_rows_json[4096];
    char source_node_tail_json[4096];
    char source_daemon_tail_json[4096];
    char source_binding_tail_json[4096];
    char source_asset_tail_json[4096];
    char source_event_tail_json[4096];
    char source_candidate_tail_json[4096];
    char source_outcome_tail_json[8192];
    char source_peer_membership_tail_json[8192];
    char source_coordination_json[4096];
    char source_peer_rows_json[8192];
    char source_coverage_json[1024];
    size_t source_node_count = 0;
    size_t source_daemon_count = 0;
    size_t source_binding_count = 0;
    size_t source_asset_count = 0;
    size_t source_event_count = 0;
    size_t source_candidate_count = 0;
    size_t source_outcome_count = 0;
    size_t source_owner_link_count = 0;
    size_t source_peer_membership_count = 0;
    size_t source_graph_node_count = 0;
    size_t source_graph_edge_count = 0;
    char source_query_err[96];
    int db_first_ready;
    int fallback_active;
    int n;

    if (err && err_cap > 0) err[0] = '\0';
    if (!out || out_cap == 0)
        return -1;

    if (yai_session_resolve_current_workspace(&info, status, sizeof(status), bind_err, sizeof(bind_err)) != 0 ||
        strcmp(status, "active") != 0)
    {
        if (err && err_cap > 0)
            (void)snprintf(err, err_cap, "%s", bind_err[0] ? bind_err : "workspace_not_active");
        return -1;
    }

    (void)snprintf(qf, sizeof(qf), "%s", (query_family && query_family[0]) ? query_family : "workspace");
    if (strcmp(qf, "workspace") != 0 &&
        strcmp(qf, "governance") != 0 &&
        strcmp(qf, "events") != 0 &&
        strcmp(qf, "evidence") != 0 &&
        strcmp(qf, "enforcement") != 0 &&
        strcmp(qf, "authority") != 0 &&
        strcmp(qf, "artifacts") != 0 &&
        strcmp(qf, "graph") != 0 &&
        strcmp(qf, "graph.workspace") != 0 &&
        strcmp(qf, "graph.governance") != 0 &&
        strcmp(qf, "graph.decision") != 0 &&
        strcmp(qf, "graph.artifact") != 0 &&
        strcmp(qf, "graph.authority") != 0 &&
        strcmp(qf, "graph.evidence") != 0 &&
        strcmp(qf, "graph.lineage") != 0 &&
        strcmp(qf, "graph.recent") != 0 &&
        strcmp(qf, "source") != 0 &&
        strcmp(qf, "source.peer") != 0 &&
        strcmp(qf, "source.coverage") != 0 &&
        strcmp(qf, "source.conflicts") != 0 &&
        strcmp(qf, "lifecycle") != 0)
    {
        if (err && err_cap > 0)
            (void)snprintf(err, err_cap, "%s", "unsupported_query_family");
        return -1;
    }

    yai_workspace_read_event_evidence_index(&info,
                                            sink_last_event_ref,
                                            sizeof(sink_last_event_ref),
                                            sink_last_decision_ref,
                                            sizeof(sink_last_decision_ref),
                                            sink_last_evidence_ref,
                                            sizeof(sink_last_evidence_ref),
                                            sink_event_store_ref,
                                            sizeof(sink_event_store_ref),
                                            sink_decision_store_ref,
                                            sizeof(sink_decision_store_ref),
                                            sink_evidence_store_ref,
                                            sizeof(sink_evidence_store_ref));
    yai_workspace_read_enforcement_index(&info,
                                         enforce_last_outcome_ref,
                                         sizeof(enforce_last_outcome_ref),
                                         enforce_last_linkage_ref,
                                         sizeof(enforce_last_linkage_ref),
                                         enforce_materialization_status,
                                         sizeof(enforce_materialization_status),
                                         enforce_missing_fields,
                                         sizeof(enforce_missing_fields),
                                         enforce_outcome_store_ref,
                                         sizeof(enforce_outcome_store_ref),
                                         enforce_linkage_store_ref,
                                         sizeof(enforce_linkage_store_ref));
    yai_workspace_read_governance_index(&info,
                                        gov_last_object_ref,
                                        sizeof(gov_last_object_ref),
                                        gov_last_lifecycle_ref,
                                        sizeof(gov_last_lifecycle_ref),
                                        gov_last_attachment_ref,
                                        sizeof(gov_last_attachment_ref),
                                        gov_object_store_ref,
                                        sizeof(gov_object_store_ref),
                                        gov_lifecycle_store_ref,
                                        sizeof(gov_lifecycle_store_ref),
                                        gov_attachment_store_ref,
                                        sizeof(gov_attachment_store_ref));
    yai_workspace_read_authority_artifact_indexes(&info,
                                                  authority_last_ref,
                                                  sizeof(authority_last_ref),
                                                  authority_resolution_ref,
                                                  sizeof(authority_resolution_ref),
                                                  artifact_last_ref,
                                                  sizeof(artifact_last_ref),
                                                  artifact_linkage_ref,
                                                  sizeof(artifact_linkage_ref),
                                                  authority_store_ref,
                                                  sizeof(authority_store_ref),
                                                  artifact_store_ref,
                                                  sizeof(artifact_store_ref));
    (void)yai_storage_bridge_last_refs(info.ws_id,
                                            brain_graph_node_ref,
                                            sizeof(brain_graph_node_ref),
                                            brain_graph_edge_ref,
                                            sizeof(brain_graph_edge_ref),
                                            brain_transient_state_ref,
                                            sizeof(brain_transient_state_ref),
                                            brain_transient_working_set_ref,
                                            sizeof(brain_transient_working_set_ref),
                                            brain_graph_store_ref,
                                            sizeof(brain_graph_store_ref),
                                            brain_transient_store_ref,
                                            sizeof(brain_transient_store_ref));
    (void)yai_workspace_graph_paths(info.ws_id,
                                    graph_nodes_log,
                                    sizeof(graph_nodes_log),
                                    graph_edges_log,
                                    sizeof(graph_edges_log),
                                    graph_index_path,
                                    sizeof(graph_index_path));
    yai_graph_load_index(graph_index_path,
                         graph_node_classes,
                         sizeof(graph_node_classes),
                         graph_edge_classes,
                         sizeof(graph_edge_classes),
                         graph_updated_at,
                         sizeof(graph_updated_at),
                         &graph_node_count,
                         &graph_edge_count);
    if (yai_graph_rows_from_csv(graph_node_classes, "node_class", graph_node_rows_json, sizeof(graph_node_rows_json), 24) != 0)
        (void)snprintf(graph_node_rows_json, sizeof(graph_node_rows_json), "%s", "");
    if (yai_graph_rows_from_csv(graph_edge_classes, "edge_class", graph_edge_rows_json, sizeof(graph_edge_rows_json), 24) != 0)
        (void)snprintf(graph_edge_rows_json, sizeof(graph_edge_rows_json), "%s", "");
    graph_query_summary_json[0] = '\0';
    graph_query_err[0] = '\0';
    if (yai_graph_query_summary(info.ws_id,
                                graph_query_summary_json,
                                sizeof(graph_query_summary_json),
                                graph_query_err,
                                sizeof(graph_query_err)) != 0)
    {
        (void)snprintf(graph_query_summary_json,
                       sizeof(graph_query_summary_json),
                       "{\"workspace_id\":\"%s\",\"error\":\"%s\"}",
                       info.ws_id,
                       graph_query_err[0] ? graph_query_err : "graph_query_unavailable");
    }
    source_node_tail_json[0] = '\0';
    source_daemon_tail_json[0] = '\0';
    source_binding_tail_json[0] = '\0';
    source_asset_tail_json[0] = '\0';
    source_event_tail_json[0] = '\0';
    source_candidate_tail_json[0] = '\0';
    source_outcome_tail_json[0] = '\0';
    source_peer_membership_tail_json[0] = '\0';
    source_coordination_json[0] = '\0';
    source_peer_rows_json[0] = '\0';
    source_coverage_json[0] = '\0';
    source_query_err[0] = '\0';
    (void)yai_data_query_count(info.ws_id, "source_node", &source_node_count, source_query_err, sizeof(source_query_err));
    (void)yai_data_query_count(info.ws_id, "source_daemon_instance", &source_daemon_count, source_query_err, sizeof(source_query_err));
    (void)yai_data_query_count(info.ws_id, "source_binding", &source_binding_count, source_query_err, sizeof(source_query_err));
    (void)yai_data_query_count(info.ws_id, "source_asset", &source_asset_count, source_query_err, sizeof(source_query_err));
    (void)yai_data_query_count(info.ws_id, "source_acquisition_event", &source_event_count, source_query_err, sizeof(source_query_err));
    (void)yai_data_query_count(info.ws_id, "source_evidence_candidate", &source_candidate_count, source_query_err, sizeof(source_query_err));
    (void)yai_data_query_count(info.ws_id, "source_ingest_outcome", &source_outcome_count, source_query_err, sizeof(source_query_err));
    (void)yai_data_query_count(info.ws_id, "source_owner_link", &source_owner_link_count, source_query_err, sizeof(source_query_err));
    (void)yai_data_query_count(info.ws_id, "workspace_peer_membership", &source_peer_membership_count, source_query_err, sizeof(source_query_err));
    (void)yai_data_query_tail_json(info.ws_id, "source_node", 3, source_node_tail_json, sizeof(source_node_tail_json), source_query_err, sizeof(source_query_err));
    (void)yai_data_query_tail_json(info.ws_id, "source_daemon_instance", 3, source_daemon_tail_json, sizeof(source_daemon_tail_json), source_query_err, sizeof(source_query_err));
    (void)yai_data_query_tail_json(info.ws_id, "source_binding", 3, source_binding_tail_json, sizeof(source_binding_tail_json), source_query_err, sizeof(source_query_err));
    (void)yai_data_query_tail_json(info.ws_id, "source_asset", 3, source_asset_tail_json, sizeof(source_asset_tail_json), source_query_err, sizeof(source_query_err));
    (void)yai_data_query_tail_json(info.ws_id, "source_acquisition_event", 3, source_event_tail_json, sizeof(source_event_tail_json), source_query_err, sizeof(source_query_err));
    (void)yai_data_query_tail_json(info.ws_id, "source_evidence_candidate", 3, source_candidate_tail_json, sizeof(source_candidate_tail_json), source_query_err, sizeof(source_query_err));
    (void)yai_data_query_tail_json(info.ws_id, "source_ingest_outcome", 5, source_outcome_tail_json, sizeof(source_outcome_tail_json), source_query_err, sizeof(source_query_err));
    (void)yai_data_query_tail_json(info.ws_id, "workspace_peer_membership", 3, source_peer_membership_tail_json, sizeof(source_peer_membership_tail_json), source_query_err, sizeof(source_query_err));
    if (yai_owner_peer_registry_workspace_summary_json(info.ws_id,
                                                       source_coordination_json,
                                                       sizeof(source_coordination_json),
                                                       source_query_err,
                                                       sizeof(source_query_err)) != 0)
    {
        (void)snprintf(source_coordination_json,
                       sizeof(source_coordination_json),
                       "{\"workspace_id\":\"%s\",\"peer_count\":0,\"states\":{\"ready\":0,\"degraded\":0,\"disconnected\":0,\"stale\":0},\"backlog\":{\"queued\":0,\"retry_due\":0,\"failed\":0},\"integrity\":{\"replay_detected\":0,\"overlap_detected\":0,\"conflict_detected\":0,\"ordering_late\":0,\"review_required\":0},\"coverage\":{\"scope_count\":0,\"distinct\":0,\"overlap\":0,\"gap\":0},\"scheduling_state\":\"unknown\",\"peers\":[]}",
                       info.ws_id);
    }
    if (yai_owner_peer_registry_workspace_peer_rows_json(info.ws_id,
                                                         source_peer_rows_json,
                                                         sizeof(source_peer_rows_json),
                                                         source_query_err,
                                                         sizeof(source_query_err)) != 0)
    {
        (void)snprintf(source_peer_rows_json, sizeof(source_peer_rows_json), "[]");
    }
    if (yai_owner_peer_registry_workspace_coverage_summary_json(info.ws_id,
                                                                source_coverage_json,
                                                                sizeof(source_coverage_json),
                                                                source_query_err,
                                                                sizeof(source_query_err)) != 0)
    {
        (void)snprintf(source_coverage_json,
                       sizeof(source_coverage_json),
                       "{\"workspace_id\":\"%s\",\"peer_count\":0,\"coverage_scope_count\":0,\"coverage_distinct_count\":0,\"overlap_count\":0,\"gap_count\":0}",
                       info.ws_id);
    }
    (void)yai_graph_materialization_workspace_source_counts(info.ws_id,
                                                            &source_graph_node_count,
                                                            &source_graph_edge_count);
    qf_model = (strncmp(qf, "graph.", 6) == 0) ? "graph" : qf;
    yai_workspace_db_first_read_model(qf_model,
                                      sink_last_event_ref,
                                      sink_last_decision_ref,
                                      sink_last_evidence_ref,
                                      gov_last_object_ref,
                                      gov_last_lifecycle_ref,
                                      authority_last_ref,
                                      artifact_last_ref,
                                      brain_graph_node_ref,
                                      brain_graph_edge_ref,
                                      enforce_last_outcome_ref,
                                      enforce_last_linkage_ref,
                                      enforce_materialization_status,
                                      read_primary_source,
                                      sizeof(read_primary_source),
                                      read_fallback_reason,
                                      sizeof(read_fallback_reason),
                                      &db_first_ready,
                                      &fallback_active);

    if (strcmp(qf, "lifecycle") == 0)
    {
        if (yai_lifecycle_render_policy_rows_json(lifecycle_rows_json, sizeof(lifecycle_rows_json)) != 0)
        {
            if (err && err_cap > 0)
                (void)snprintf(err, err_cap, "%s", "lifecycle_policy_render_failed");
            return -1;
        }
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"lifecycle\","
                     "\"result_shape\":\"table\","
                     "\"workspace_id\":\"%s\","
                     "\"model\":{\"matrix_ref\":\"%s\",\"bounded_hot_set\":true,\"workspace_scoped_partitioning\":true,\"transient_authoritative\":false},"
                     "\"columns\":[\"data_class\",\"tier\",\"retention_profile\",\"lineage_required\",\"compactable\",\"archive_eligible\"],"
                     "\"rows\":[%s],"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"data_plane_lifecycle_policy\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false}"
                     "}",
                     info.ws_id,
                     yai_lifecycle_policy_matrix_ref(),
                     lifecycle_rows_json,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "governance") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"governance\","
                     "\"result_shape\":\"table\","
                     "\"workspace_id\":\"%s\","
                     "\"columns\":[\"object_ref\",\"lifecycle_ref\",\"attachment_ref\",\"attachment_count\",\"active_stack_ref\"],"
                     "\"rows\":[[\"%s\",\"%s\",\"%s\",%d,\"%s\"]],"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"%s\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                     "\"stores\":{\"object_store_ref\":\"%s\",\"lifecycle_store_ref\":\"%s\",\"attachment_store_ref\":\"%s\"}"
                     "}",
                     info.ws_id,
                     gov_last_object_ref,
                     gov_last_lifecycle_ref,
                     gov_last_attachment_ref,
                     info.policy_attachment_count,
                     info.effective_stack_ref,
                     read_primary_source,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason,
                     gov_object_store_ref,
                     gov_lifecycle_store_ref,
                     gov_attachment_store_ref);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "events") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"events\","
                     "\"result_shape\":\"timeline\","
                     "\"workspace_id\":\"%s\","
                     "\"items\":["
                     "{\"event_ref\":\"%s\",\"decision_ref\":\"%s\",\"evidence_ref\":\"%s\",\"trace_ref\":\"%s\",\"effect\":\"%s\"}"
                     "],"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"%s\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                     "\"stores\":{\"event_store_ref\":\"%s\",\"decision_store_ref\":\"%s\",\"evidence_store_ref\":\"%s\"}"
                     "}",
                     info.ws_id,
                     sink_last_event_ref,
                     sink_last_decision_ref,
                     sink_last_evidence_ref,
                     info.last_resolution_trace_ref,
                     info.last_effect_summary,
                     read_primary_source,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason,
                     sink_event_store_ref,
                     sink_decision_store_ref,
                     sink_evidence_store_ref);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "evidence") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"evidence\","
                     "\"result_shape\":\"detail_record\","
                     "\"workspace_id\":\"%s\","
                     "\"record\":{\"decision_ref\":\"%s\",\"evidence_ref\":\"%s\",\"authority_summary\":\"%s\",\"evidence_summary\":\"%s\",\"trace_ref\":\"%s\"},"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"%s\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                     "\"stores\":{\"decision_store_ref\":\"%s\",\"evidence_store_ref\":\"%s\"}"
                     "}",
                     info.ws_id,
                     sink_last_decision_ref,
                     sink_last_evidence_ref,
                     info.last_authority_summary,
                     info.last_evidence_summary,
                     info.last_resolution_trace_ref,
                     read_primary_source,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason,
                     sink_decision_store_ref,
                     sink_evidence_store_ref);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "enforcement") == 0)
    {
        char enforcement_outcome_tail_json[2048];
        char enforcement_linkage_tail_json[2048];
        char enforcement_outcome_latest_json[1024];
        char enforcement_authority_constraints[256];
        char enforcement_authority_decision[64];
        char authority_requirement_coverage_json[1024];
        int all_requirements_covered = 0;
        size_t enforcement_outcome_count = 0;
        size_t enforcement_linkage_count = 0;
        char enforcement_query_err[96];
        enforcement_outcome_tail_json[0] = '\0';
        enforcement_linkage_tail_json[0] = '\0';
        enforcement_outcome_latest_json[0] = '\0';
        enforcement_authority_constraints[0] = '\0';
        enforcement_authority_decision[0] = '\0';
        authority_requirement_coverage_json[0] = '\0';
        enforcement_query_err[0] = '\0';
        (void)yai_data_query_tail_json(info.ws_id,
                                       "enforcement_outcome",
                                       3,
                                       enforcement_outcome_tail_json,
                                       sizeof(enforcement_outcome_tail_json),
                                       enforcement_query_err,
                                       sizeof(enforcement_query_err));
        (void)yai_data_query_tail_json(info.ws_id,
                                       "enforcement_linkage",
                                       3,
                                       enforcement_linkage_tail_json,
                                       sizeof(enforcement_linkage_tail_json),
                                       enforcement_query_err,
                                       sizeof(enforcement_query_err));
        (void)yai_data_query_tail_json(info.ws_id,
                                       "enforcement_outcome",
                                       1,
                                       enforcement_outcome_latest_json,
                                       sizeof(enforcement_outcome_latest_json),
                                       enforcement_query_err,
                                       sizeof(enforcement_query_err));
        (void)yai_data_query_count(info.ws_id, "enforcement_outcome", &enforcement_outcome_count, enforcement_query_err, sizeof(enforcement_query_err));
        (void)yai_data_query_count(info.ws_id, "enforcement_linkage", &enforcement_linkage_count, enforcement_query_err, sizeof(enforcement_query_err));
        if (enforcement_outcome_tail_json[0] == '\0') snprintf(enforcement_outcome_tail_json, sizeof(enforcement_outcome_tail_json), "[]");
        if (enforcement_linkage_tail_json[0] == '\0') snprintf(enforcement_linkage_tail_json, sizeof(enforcement_linkage_tail_json), "[]");
        if (enforcement_outcome_latest_json[0] == '\0') snprintf(enforcement_outcome_latest_json, sizeof(enforcement_outcome_latest_json), "[]");

        (void)yai_session_extract_json_string(enforcement_outcome_latest_json,
                                              "authority_constraints",
                                              enforcement_authority_constraints,
                                              sizeof(enforcement_authority_constraints));
        (void)yai_session_extract_json_string(enforcement_outcome_latest_json,
                                              "authority_decision",
                                              enforcement_authority_decision,
                                              sizeof(enforcement_authority_decision));
        if (yai_authority_requirement_coverage_from_csv(enforcement_authority_constraints,
                                                        enforcement_authority_decision,
                                                        authority_requirement_coverage_json,
                                                        sizeof(authority_requirement_coverage_json),
                                                        24,
                                                        &all_requirements_covered) != 0)
        {
            authority_requirement_coverage_json[0] = '\0';
            all_requirements_covered = 0;
        }
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"enforcement\","
                     "\"result_shape\":\"detail_record\","
                     "\"workspace_id\":\"%s\","
                     "\"record\":{\"outcome_ref\":\"%s\",\"linkage_ref\":\"%s\",\"materialization_status\":\"%s\",\"missing_fields\":\"%s\",\"event_ref\":\"%s\",\"decision_ref\":\"%s\",\"evidence_ref\":\"%s\"},"
                     "\"records\":{\"enforcement_outcome_count\":%zu,\"enforcement_linkage_count\":%zu,\"latest_enforcement_outcome\":%s,\"latest_enforcement_linkage\":%s},"
                     "\"authority_requirement_coverage\":{\"all_covered\":%s,\"authority_decision\":\"%s\",\"requirements\":[%s]},"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"%s\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                     "\"stores\":{\"outcome_store_ref\":\"%s\",\"linkage_store_ref\":\"%s\"}"
                     "}",
                     info.ws_id,
                     enforce_last_outcome_ref,
                     enforce_last_linkage_ref,
                     enforce_materialization_status[0] ? enforce_materialization_status : "unknown",
                     enforce_missing_fields,
                     sink_last_event_ref,
                     sink_last_decision_ref,
                     sink_last_evidence_ref,
                     enforcement_outcome_count,
                     enforcement_linkage_count,
                     enforcement_outcome_tail_json,
                     enforcement_linkage_tail_json,
                     all_requirements_covered ? "true" : "false",
                     enforcement_authority_decision[0] ? enforcement_authority_decision : "unknown",
                     authority_requirement_coverage_json,
                     read_primary_source,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason,
                     enforce_outcome_store_ref,
                     enforce_linkage_store_ref);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "authority") == 0)
    {
        char authority_tail_json[2048];
        char authority_resolution_tail_json[2048];
        char enforcement_outcome_tail_json[2048];
        char enforcement_linkage_tail_json[2048];
        char enforcement_outcome_latest_json[1024];
        char enforcement_linkage_latest_json[1024];
        char enforcement_authority_constraints[256];
        char enforcement_authority_decision[64];
        int constraint_coverage = 0;
        size_t authority_count = 0;
        size_t authority_resolution_count = 0;
        size_t enforcement_outcome_count = 0;
        size_t enforcement_linkage_count = 0;
        char authority_query_err[96];
        authority_tail_json[0] = '\0';
        authority_resolution_tail_json[0] = '\0';
        enforcement_outcome_tail_json[0] = '\0';
        enforcement_linkage_tail_json[0] = '\0';
        enforcement_outcome_latest_json[0] = '\0';
        enforcement_linkage_latest_json[0] = '\0';
        enforcement_authority_constraints[0] = '\0';
        enforcement_authority_decision[0] = '\0';
        authority_query_err[0] = '\0';
        (void)yai_data_query_tail_json(info.ws_id,
                                       "authority",
                                       3,
                                       authority_tail_json,
                                       sizeof(authority_tail_json),
                                       authority_query_err,
                                       sizeof(authority_query_err));
        (void)yai_data_query_tail_json(info.ws_id,
                                       "authority_resolution",
                                       3,
                                       authority_resolution_tail_json,
                                       sizeof(authority_resolution_tail_json),
                                       authority_query_err,
                                       sizeof(authority_query_err));
        (void)yai_data_query_count(info.ws_id, "authority", &authority_count, authority_query_err, sizeof(authority_query_err));
        (void)yai_data_query_count(info.ws_id, "authority_resolution", &authority_resolution_count, authority_query_err, sizeof(authority_query_err));
        (void)yai_data_query_count(info.ws_id, "enforcement_outcome", &enforcement_outcome_count, authority_query_err, sizeof(authority_query_err));
        (void)yai_data_query_count(info.ws_id, "enforcement_linkage", &enforcement_linkage_count, authority_query_err, sizeof(authority_query_err));
        (void)yai_data_query_tail_json(info.ws_id,
                                       "enforcement_outcome",
                                       3,
                                       enforcement_outcome_tail_json,
                                       sizeof(enforcement_outcome_tail_json),
                                       authority_query_err,
                                       sizeof(authority_query_err));
        (void)yai_data_query_tail_json(info.ws_id,
                                       "enforcement_linkage",
                                       3,
                                       enforcement_linkage_tail_json,
                                       sizeof(enforcement_linkage_tail_json),
                                       authority_query_err,
                                       sizeof(authority_query_err));
        (void)yai_data_query_tail_json(info.ws_id,
                                       "enforcement_outcome",
                                       1,
                                       enforcement_outcome_latest_json,
                                       sizeof(enforcement_outcome_latest_json),
                                       authority_query_err,
                                       sizeof(authority_query_err));
        (void)yai_data_query_tail_json(info.ws_id,
                                       "enforcement_linkage",
                                       1,
                                       enforcement_linkage_latest_json,
                                       sizeof(enforcement_linkage_latest_json),
                                       authority_query_err,
                                       sizeof(authority_query_err));
        if (authority_tail_json[0] == '\0') snprintf(authority_tail_json, sizeof(authority_tail_json), "[]");
        if (authority_resolution_tail_json[0] == '\0') snprintf(authority_resolution_tail_json, sizeof(authority_resolution_tail_json), "[]");
        if (enforcement_outcome_tail_json[0] == '\0') snprintf(enforcement_outcome_tail_json, sizeof(enforcement_outcome_tail_json), "[]");
        if (enforcement_linkage_tail_json[0] == '\0') snprintf(enforcement_linkage_tail_json, sizeof(enforcement_linkage_tail_json), "[]");
        if (enforcement_outcome_latest_json[0] == '\0') snprintf(enforcement_outcome_latest_json, sizeof(enforcement_outcome_latest_json), "[]");
        if (enforcement_linkage_latest_json[0] == '\0') snprintf(enforcement_linkage_latest_json, sizeof(enforcement_linkage_latest_json), "[]");
        (void)yai_session_extract_json_string(enforcement_outcome_latest_json,
                                              "authority_constraints",
                                              enforcement_authority_constraints,
                                              sizeof(enforcement_authority_constraints));
        (void)yai_session_extract_json_string(enforcement_outcome_latest_json,
                                              "authority_decision",
                                              enforcement_authority_decision,
                                              sizeof(enforcement_authority_decision));
        if (enforcement_authority_constraints[0] && enforcement_authority_decision[0]) {
            constraint_coverage = 1;
        }
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"authority\","
                     "\"result_shape\":\"table\","
                     "\"workspace_id\":\"%s\","
                     "\"columns\":[\"authority_ref\",\"resolution_ref\",\"authority_summary\",\"governance_refs\"],"
                     "\"rows\":[[\"%s\",\"%s\",\"%s\",\"%s\"]],"
                     "\"records\":{\"authority_count\":%zu,\"authority_resolution_count\":%zu,\"enforcement_outcome_count\":%zu,\"enforcement_linkage_count\":%zu,"
                     "\"latest_authority\":%s,\"latest_authority_resolution\":%s,\"latest_enforcement_outcome\":%s,\"latest_enforcement_linkage\":%s},"
                     "\"correlation\":{\"constraint_coverage\":%s,\"authority_decision\":\"%s\",\"authority_constraints\":\"%s\"},"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"%s\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                     "\"stores\":{\"authority_store_ref\":\"%s\"}"
                     "}",
                     info.ws_id,
                     authority_last_ref,
                     authority_resolution_ref,
                     info.last_authority_summary,
                     info.policy_attachments_csv,
                     authority_count,
                     authority_resolution_count,
                     enforcement_outcome_count,
                     enforcement_linkage_count,
                     authority_tail_json,
                     authority_resolution_tail_json,
                     enforcement_outcome_tail_json,
                     enforcement_linkage_tail_json,
                     constraint_coverage ? "true" : "false",
                     enforcement_authority_decision[0] ? enforcement_authority_decision : "unknown",
                     enforcement_authority_constraints[0] ? enforcement_authority_constraints : "none",
                     read_primary_source,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason,
                     authority_store_ref);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "artifacts") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"artifacts\","
                     "\"result_shape\":\"table\","
                     "\"workspace_id\":\"%s\","
                     "\"columns\":[\"artifact_ref\",\"artifact_linkage_ref\",\"decision_ref\",\"evidence_ref\"],"
                     "\"rows\":[[\"%s\",\"%s\",\"%s\",\"%s\"]],"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"%s\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                     "\"stores\":{\"artifact_store_ref\":\"%s\"}"
                     "}",
                     info.ws_id,
                     artifact_last_ref,
                     artifact_linkage_ref,
                     sink_last_decision_ref,
                     sink_last_evidence_ref,
                     read_primary_source,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason,
                     artifact_store_ref);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "source") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"source\","
                     "\"result_shape\":\"source_plane_summary\","
                     "\"workspace_id\":\"%s\","
                     "\"summary\":{\"source_node_count\":%zu,\"source_daemon_instance_count\":%zu,\"source_binding_count\":%zu,"
                     "\"source_asset_count\":%zu,\"source_acquisition_event_count\":%zu,\"source_evidence_candidate_count\":%zu,\"source_ingest_outcome_count\":%zu,"
                     "\"source_owner_link_count\":%zu,\"workspace_peer_membership_count\":%zu,\"source_graph_node_count\":%zu,\"source_graph_edge_count\":%zu},"
                     "\"records\":{\"source_nodes\":%s,\"source_daemon_instances\":%s,\"source_bindings\":%s,"
                     "\"source_assets\":%s,\"source_acquisition_events\":%s,\"source_evidence_candidates\":%s,\"source_ingest_outcomes\":%s,\"workspace_peer_memberships\":%s},"
                     "\"coordination\":%s,"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"data_plane_persisted_records\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                     "\"graph\":{\"workspace_summary\":%s}"
                     "}",
                     info.ws_id,
                     source_node_count,
                     source_daemon_count,
                     source_binding_count,
                     source_asset_count,
                     source_event_count,
                     source_candidate_count,
                     source_outcome_count,
                     source_owner_link_count,
                     source_peer_membership_count,
                     source_graph_node_count,
                     source_graph_edge_count,
                     source_node_tail_json[0] ? source_node_tail_json : "[]",
                     source_daemon_tail_json[0] ? source_daemon_tail_json : "[]",
                     source_binding_tail_json[0] ? source_binding_tail_json : "[]",
                     source_asset_tail_json[0] ? source_asset_tail_json : "[]",
                     source_event_tail_json[0] ? source_event_tail_json : "[]",
                     source_candidate_tail_json[0] ? source_candidate_tail_json : "[]",
                     source_outcome_tail_json[0] ? source_outcome_tail_json : "[]",
                     source_peer_membership_tail_json[0] ? source_peer_membership_tail_json : "[]",
                     source_coordination_json,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason,
                     graph_query_summary_json);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "source.peer") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"source.peer\","
                     "\"result_shape\":\"source_peer_inspect\","
                     "\"workspace_id\":\"%s\","
                     "\"summary\":{\"peer_count\":%zu,\"workspace_peer_membership_count\":%zu},"
                     "\"rows\":%s,"
                     "\"coordination\":%s,"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"data_plane_persisted_records\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false}"
                     "}",
                     info.ws_id,
                     source_peer_membership_count,
                     source_peer_membership_count,
                     source_peer_rows_json[0] ? source_peer_rows_json : "[]",
                     source_coordination_json,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "source.coverage") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"source.coverage\","
                     "\"result_shape\":\"source_peer_coverage_summary\","
                     "\"workspace_id\":\"%s\","
                     "\"coverage\":%s,"
                     "\"coordination\":%s,"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"data_plane_persisted_records\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false}"
                     "}",
                     info.ws_id,
                     source_coverage_json,
                     source_coordination_json,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "source.conflicts") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"source.conflicts\","
                     "\"result_shape\":\"source_peer_conflict_summary\","
                     "\"workspace_id\":\"%s\","
                     "\"summary\":{\"source_ingest_outcome_count\":%zu},"
                     "\"rows\":%s,"
                     "\"coordination\":%s,"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"data_plane_persisted_records\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false}"
                     "}",
                     info.ws_id,
                     source_outcome_count,
                     source_outcome_tail_json[0] ? source_outcome_tail_json : "[]",
                     source_coordination_json,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "graph.workspace") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"graph.workspace\","
                     "\"result_shape\":\"graph_neighborhood_table\","
                     "\"workspace_id\":\"%s\","
                     "\"summary\":{\"graph_node_count\":%ld,\"graph_edge_count\":%ld,\"updated_at\":\"%s\","
                     "\"last_graph_node_ref\":\"%s\",\"last_graph_edge_ref\":\"%s\",\"graph_truth_authoritative\":true,"
                     "\"source_graph_node_count\":%zu,\"source_graph_edge_count\":%zu},"
                     "\"graph_query_summary\":%s,"
                     "\"columns\":[\"edge_class\",\"kind\"],"
                     "\"rows\":[%s],"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"graph_truth_store\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                     "\"stores\":{\"graph_nodes_ref\":\"%s\",\"graph_edges_ref\":\"%s\",\"graph_index_ref\":\"%s\"}"
                     "}",
                     info.ws_id,
                     graph_node_count,
                     graph_edge_count,
                     graph_updated_at[0] ? graph_updated_at : "unknown",
                     brain_graph_node_ref,
                     brain_graph_edge_ref,
                     source_graph_node_count,
                     source_graph_edge_count,
                     graph_query_summary_json,
                     graph_edge_rows_json,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason,
                     graph_nodes_log,
                     graph_edges_log,
                     graph_index_path);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "graph.governance") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"graph.governance\","
                     "\"result_shape\":\"summary_card\","
                     "\"workspace_id\":\"%s\","
                     "\"summary\":{\"governance_ref\":\"%s\",\"lifecycle_ref\":\"%s\",\"attachment_ref\":\"%s\","
                     "\"graph_edge_classes\":[%s],\"graph_node_count\":%ld,\"graph_edge_count\":%ld},"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"graph_truth_store\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                     "\"stores\":{\"graph_index_ref\":\"%s\",\"governance_object_store_ref\":\"%s\",\"governance_lifecycle_store_ref\":\"%s\"}"
                     "}",
                     info.ws_id,
                     gov_last_object_ref,
                     gov_last_lifecycle_ref,
                     gov_last_attachment_ref,
                     graph_edge_rows_json,
                     graph_node_count,
                     graph_edge_count,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason,
                     graph_index_path,
                     gov_object_store_ref,
                     gov_lifecycle_store_ref);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "graph.decision") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"graph.decision\","
                     "\"result_shape\":\"detail_record\","
                     "\"workspace_id\":\"%s\","
                     "\"detail\":{\"decision_ref\":\"%s\",\"event_ref\":\"%s\",\"evidence_ref\":\"%s\","
                     "\"governance_ref\":\"%s\",\"authority_ref\":\"%s\",\"artifact_ref\":\"%s\","
                     "\"relation_classes\":[%s],\"last_graph_node_ref\":\"%s\",\"last_graph_edge_ref\":\"%s\"},"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"graph_truth_store\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                     "\"stores\":{\"graph_nodes_ref\":\"%s\",\"graph_edges_ref\":\"%s\"}"
                     "}",
                     info.ws_id,
                     sink_last_decision_ref,
                     sink_last_event_ref,
                     sink_last_evidence_ref,
                     gov_last_object_ref,
                     authority_last_ref,
                     artifact_last_ref,
                     graph_edge_rows_json,
                     brain_graph_node_ref,
                     brain_graph_edge_ref,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason,
                     graph_nodes_log,
                     graph_edges_log);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "graph.artifact") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"graph.artifact\","
                     "\"result_shape\":\"summary_card\","
                     "\"workspace_id\":\"%s\","
                     "\"summary\":{\"artifact_ref\":\"%s\",\"artifact_linkage_ref\":\"%s\",\"decision_ref\":\"%s\",\"evidence_ref\":\"%s\","
                     "\"relation_classes\":[%s]},"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"graph_truth_store\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                     "\"stores\":{\"artifact_store_ref\":\"%s\",\"graph_index_ref\":\"%s\"}"
                     "}",
                     info.ws_id,
                     artifact_last_ref,
                     artifact_linkage_ref,
                     sink_last_decision_ref,
                     sink_last_evidence_ref,
                     graph_edge_rows_json,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason,
                     artifact_store_ref,
                     graph_index_path);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "graph.authority") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"graph.authority\","
                     "\"result_shape\":\"summary_card\","
                     "\"workspace_id\":\"%s\","
                     "\"summary\":{\"authority_ref\":\"%s\",\"authority_resolution_ref\":\"%s\",\"decision_ref\":\"%s\","
                     "\"governance_ref\":\"%s\",\"relation_classes\":[%s]},"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"graph_truth_store\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                     "\"stores\":{\"authority_store_ref\":\"%s\",\"graph_index_ref\":\"%s\"}"
                     "}",
                     info.ws_id,
                     authority_last_ref,
                     authority_resolution_ref,
                     sink_last_decision_ref,
                     gov_last_object_ref,
                     graph_edge_rows_json,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason,
                     authority_store_ref,
                     graph_index_path);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "graph.evidence") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"graph.evidence\","
                     "\"result_shape\":\"summary_card\","
                     "\"workspace_id\":\"%s\","
                     "\"summary\":{\"evidence_ref\":\"%s\",\"decision_ref\":\"%s\",\"artifact_ref\":\"%s\","
                     "\"relation_classes\":[%s]},"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"graph_truth_store\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                     "\"stores\":{\"evidence_store_ref\":\"%s\",\"graph_index_ref\":\"%s\"}"
                     "}",
                     info.ws_id,
                     sink_last_evidence_ref,
                     sink_last_decision_ref,
                     artifact_last_ref,
                     graph_edge_rows_json,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason,
                     sink_evidence_store_ref,
                     graph_index_path);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "graph.lineage") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"graph.lineage\","
                     "\"result_shape\":\"timeline\","
                     "\"workspace_id\":\"%s\","
                     "\"items\":["
                     "{\"kind\":\"governance_lineage\",\"ref\":\"%s\",\"lifecycle_ref\":\"%s\"},"
                     "{\"kind\":\"decision_evidence_linkage\",\"decision_ref\":\"%s\",\"evidence_ref\":\"%s\"},"
                     "{\"kind\":\"graph_anchor\",\"graph_node_ref\":\"%s\",\"graph_edge_ref\":\"%s\",\"updated_at\":\"%s\"}"
                     "],"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"graph_truth_store\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                     "\"stores\":{\"graph_index_ref\":\"%s\"}"
                     "}",
                     info.ws_id,
                     gov_last_object_ref,
                     gov_last_lifecycle_ref,
                     sink_last_decision_ref,
                     sink_last_evidence_ref,
                     brain_graph_node_ref,
                     brain_graph_edge_ref,
                     graph_updated_at[0] ? graph_updated_at : "unknown",
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason,
                     graph_index_path);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "graph.recent") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"graph.recent\","
                     "\"result_shape\":\"table\","
                     "\"workspace_id\":\"%s\","
                     "\"columns\":[\"node_or_edge_class\",\"kind\"],"
                     "\"rows\":[%s%s%s],"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"graph_truth_store\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                     "\"stores\":{\"graph_nodes_ref\":\"%s\",\"graph_edges_ref\":\"%s\",\"graph_index_ref\":\"%s\"}"
                     "}",
                     info.ws_id,
                     graph_node_rows_json,
                     (graph_node_rows_json[0] && graph_edge_rows_json[0]) ? "," : "",
                     graph_edge_rows_json,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason,
                     graph_nodes_log,
                     graph_edges_log,
                     graph_index_path);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    if (strcmp(qf, "graph") == 0)
    {
        n = snprintf(out,
                     out_cap,
                     "{"
                     "\"type\":\"yai.workspace.query.result.v1\","
                     "\"query_family\":\"graph\","
                     "\"result_shape\":\"summary_card\","
                     "\"workspace_id\":\"%s\","
                     "\"summary\":{\"last_graph_node_ref\":\"%s\",\"last_graph_edge_ref\":\"%s\",\"last_transient_state_ref\":\"%s\",\"last_transient_working_set_ref\":\"%s\",\"graph_truth_authoritative\":true,\"transient_authoritative\":false,"
                     "\"graph_node_count\":%ld,\"graph_edge_count\":%ld,\"updated_at\":\"%s\"},"
                     "\"node_classes\":[%s],"
                     "\"edge_classes\":[%s],"
                     "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"%s\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                     "\"stores\":{\"graph_store_ref\":\"%s\",\"transient_store_ref\":\"%s\",\"graph_index_ref\":\"%s\"}"
                     "}",
                     info.ws_id,
                     brain_graph_node_ref,
                     brain_graph_edge_ref,
                     brain_transient_state_ref,
                     brain_transient_working_set_ref,
                     graph_node_count,
                     graph_edge_count,
                     graph_updated_at[0] ? graph_updated_at : "unknown",
                     graph_node_rows_json,
                     graph_edge_rows_json,
                     read_primary_source,
                     db_first_ready ? "true" : "false",
                     fallback_active ? "true" : "false",
                     read_fallback_reason,
                     brain_graph_store_ref,
                     brain_transient_store_ref,
                     graph_index_path);
        return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
    }

    n = snprintf(out,
                 out_cap,
                 "{"
                 "\"type\":\"yai.workspace.query.result.v1\","
                 "\"query_family\":\"workspace\","
                 "\"result_shape\":\"summary_card\","
                 "\"workspace_id\":\"%s\","
                 "\"summary\":{\"active_stack_ref\":\"%s\",\"policy_attachments\":\"%s\",\"policy_attachment_count\":%d,\"last_effect\":\"%s\",\"last_trace_ref\":\"%s\"},"
                 "\"read_path\":{\"mode\":\"db_first\",\"primary_source\":\"%s\",\"db_first_ready\":%s,\"fallback_active\":%s,\"fallback_reason\":\"%s\",\"filesystem_primary\":false},"
                 "\"refs\":{\"governance_object_ref\":\"%s\",\"event_ref\":\"%s\",\"decision_ref\":\"%s\",\"evidence_ref\":\"%s\",\"authority_ref\":\"%s\",\"artifact_ref\":\"%s\",\"graph_node_ref\":\"%s\"}"
                 "}",
                 info.ws_id,
                 info.effective_stack_ref,
                 info.policy_attachments_csv,
                 info.policy_attachment_count,
                 info.last_effect_summary,
                 info.last_resolution_trace_ref,
                 read_primary_source,
                 db_first_ready ? "true" : "false",
                 fallback_active ? "true" : "false",
                 read_fallback_reason,
                 gov_last_object_ref,
                 sink_last_event_ref,
                 sink_last_decision_ref,
                 sink_last_evidence_ref,
                 authority_last_ref,
                 artifact_last_ref,
                 brain_graph_node_ref);
    return (n > 0 && (size_t)n < out_cap) ? 0 : -1;
}
