#define _POSIX_C_SOURCE 200809L

#include <yai/container/workspace_runtime.h>
#include "internal/control_session_internal.h"
#include <yai/supervisor/lifecycle.h>
#include <yai/container/workspace.h>
#include <yai/data/records.h>
#include <yai/data/binding.h>
#include <yai/data/query.h>
#include <yai/orchestration/runtime.h>
#include <yai/orchestration/internal/peer_registry_bridge.h>
#include <yai/graph/materialization.h>
#include <yai/graph/query.h>
#include <yai/cognition/memory.h>
#include <yai/policy/governance/policy_effects.h>
#include "cJSON.h"

#include <dirent.h>
#include <errno.h>
#include <ctype.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

#define YAI_WS_JSON_IO_CAP 262144

#define YAI_MANAGED_BEGIN "# BEGIN YAI MANAGED SHELL INTEGRATION"
#define YAI_MANAGED_END   "# END YAI MANAGED SHELL INTEGRATION"
#define YAI_POLICY_ATTACHMENTS_MAX 16

static int yai_governance_root_path(char *out, size_t out_cap, const char *rel);
static int yai_read_text(const char *path, char *out, size_t out_cap);
static int mkdir_if_missing(const char *path, mode_t mode);
int yai_governance_decision_to_audit_blob(const yai_governance_decision_t *decision, char *out, size_t out_cap);
int yai_governance_evidence_to_record_blob(const yai_governance_decision_t *decision,
                                    const yai_governance_evidence_envelope_t *evidence,
                                    char *out,
                                    size_t out_cap);

typedef struct {
    int found;
    char id[192];
    char kind[96];
    char status[48];
    char review_state[48];
    int runtime_consumable;
    int experimental;
    int deprecated;
    char attachment_modes_csv[192];
    char precedence_class[192];
    char workspace_targets_csv[256];
    char family_targets_csv[192];
    char specialization_targets_csv[192];
    char manifest_ref[MAX_PATH_LEN];
} yai_governable_object_meta_t;

#include "internal/lifecycle_policy.inc.c"

#include "internal/helpers_events.inc.c"

#include "internal/helpers_domains.inc.c"

#include "internal/lifecycle_execution.inc.c"

#include "internal/runtime_io.inc.c"

#include "internal/container_lifecycle.inc.c"

#include "internal/surface_core.inc.c"

#include "internal/surface_mutations.inc.c"

#include "internal/surface_views.inc.c"

int yai_session_record_resolution_snapshot(const char *ws_id,
                                          const yai_governance_resolution_output_t *law_out,
                                          const yai_enforcement_decision_t *enforcement_out,
                                          char *err,
                                          size_t err_cap)
{
    yai_workspace_runtime_info_t info;
    const yai_governance_effective_stack_t *stack;
    int i;
    char overlays[192];
    char event_ref[96];
    char decision_ref[96];
    char evidence_ref[96];
    char authority_last_ref[192];
    char authority_resolution_ref[192];
    char artifact_last_ref[192];
    char artifact_linkage_ref[192];
    char canonical_authority_ref[128];
    char canonical_authority_resolution_ref[128];
    char canonical_authority_record[1024];
    char canonical_authority_resolution_record[1280];
    char canonical_enforcement_outcome_ref[128];
    char canonical_enforcement_linkage_ref[128];
    char canonical_enforcement_outcome_record[1536];
    char canonical_enforcement_linkage_record[1536];
    char enforce_last_outcome_ref[192];
    char enforce_last_linkage_ref[192];
    char enforce_materialization_status[48];
    char enforce_missing_fields[192];
    char authority_requirements_csv[256];
    size_t authority_req_used = 0;
    size_t used = 0;
    int n;

    if (err && err_cap > 0)
        err[0] = '\0';
    if (!ws_id || !law_out || !enforcement_out)
        return -1;
    if (yai_session_read_workspace_info(ws_id, &info) != 0 || !info.exists)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "workspace_manifest_missing");
        return -1;
    }
    if (yai_session_enforce_workspace_scope(ws_id, err, err_cap) != 0)
        return -1;
    if (yai_workspace_bind_runtime_capabilities(ws_id, err, err_cap) != 0)
        return -1;

    stack = &law_out->decision.stack;
    snprintf(info.inferred_family, sizeof(info.inferred_family), "%s", law_out->decision.family_id);
    snprintf(info.inferred_specialization, sizeof(info.inferred_specialization), "%s", law_out->decision.specialization_id);
    info.inferred_confidence = 1.0;
    snprintf(info.effective_stack_ref, sizeof(info.effective_stack_ref), "%s", stack->stack_id);
    snprintf(info.last_effect_summary, sizeof(info.last_effect_summary), "%s", yai_governance_effect_name(law_out->decision.final_effect));
    snprintf(info.last_authority_summary, sizeof(info.last_authority_summary), "%s", stack->authority_profile);
    snprintf(info.last_evidence_summary, sizeof(info.last_evidence_summary), "%s", stack->evidence_profile);
    snprintf(info.last_resolution_trace_ref, sizeof(info.last_resolution_trace_ref), "%s", law_out->evidence.trace_id);
    snprintf(info.last_resolution_summary,
             sizeof(info.last_resolution_summary),
             "%s/%s => %s",
             law_out->decision.family_id,
             law_out->decision.specialization_id,
             yai_governance_effect_name(law_out->decision.final_effect));

    if (yai_workspace_append_event_evidence_records(ws_id,
                                                    law_out,
                                                    event_ref,
                                                    sizeof(event_ref),
                                                    decision_ref,
                                                    sizeof(decision_ref),
                                                    evidence_ref,
                                                    sizeof(evidence_ref),
                                                    err,
                                                    err_cap) != 0)
    {
        if (err && err_cap > 0 && err[0] == '\0')
            snprintf(err, err_cap, "%s", "event_evidence_sink_write_failed");
        return -1;
    }
    {
        char canonical_ref[128];
        char canonical_err[96];
        char canonical_record[768];
        (void)snprintf(canonical_record,
                       sizeof(canonical_record),
                       "{\"workspace_id\":\"%s\",\"trace_ref\":\"%s\",\"event_ref\":\"%s\",\"decision_ref\":\"%s\",\"evidence_ref\":\"%s\",\"effect\":\"%s\"}",
                       ws_id,
                       law_out->evidence.trace_id,
                       event_ref,
                       decision_ref,
                       evidence_ref,
                       yai_governance_effect_name(law_out->decision.final_effect));
        if (yai_data_records_append_event(ws_id,
                                          canonical_record,
                                          canonical_ref,
                                          sizeof(canonical_ref),
                                          canonical_err,
                                          sizeof(canonical_err)) != 0 &&
            err && err_cap > 0 && err[0] == '\0')
        {
            snprintf(err, err_cap, "%s", canonical_err[0] ? canonical_err : "canonical_event_record_append_failed");
        }
    }
    (void)snprintf(info.last_resolution_trace_ref, sizeof(info.last_resolution_trace_ref), "%s", law_out->evidence.trace_id);

    if (info.policy_attachments_csv[0])
    {
        char refs[sizeof(info.policy_attachments_csv)];
        char *tok;
        char *save = NULL;
        snprintf(refs, sizeof(refs), "%s", info.policy_attachments_csv);
        tok = strtok_r(refs, ",", &save);
        while (tok)
        {
            yai_governable_object_meta_t meta;
            yai_governable_meta_defaults(&meta);
            if (tok[0] && yai_governance_governable_object_lookup(tok, &meta))
            {
                char governance_err[96];
                if (yai_workspace_append_governance_persistence(ws_id,
                                                                &meta,
                                                                tok,
                                                                "resolution_link",
                                                                yai_workspace_attachment_state_for_meta(&meta, 1),
                                                                yai_governance_apply_eligibility_for_meta(&meta),
                                                                "runtime_resolved",
                                                                "none",
                                                                "",
                                                                event_ref,
                                                                decision_ref,
                                                                evidence_ref,
                                                                governance_err,
                                                                sizeof(governance_err)) != 0)
                {
                    if (err && err_cap > 0)
                        snprintf(err, err_cap, "%s", governance_err[0] ? governance_err : "governance_persistence_link_failed");
                    return -1;
                }
            }
            tok = strtok_r(NULL, ",", &save);
        }
    }

    {
        char authority_artifact_err[96];
        if (yai_workspace_append_authority_artifact_persistence(ws_id,
                                                                law_out,
                                                                info.policy_attachments_csv,
                                                                event_ref,
                                                                decision_ref,
                                                                evidence_ref,
                                                                authority_artifact_err,
                                                                sizeof(authority_artifact_err)) != 0)
        {
            if (err && err_cap > 0)
                snprintf(err, err_cap, "%s", authority_artifact_err[0] ? authority_artifact_err : "authority_artifact_persistence_failed");
            return -1;
        }
    }
    yai_workspace_read_authority_artifact_indexes(&info,
                                                  authority_last_ref,
                                                  sizeof(authority_last_ref),
                                                  authority_resolution_ref,
                                                  sizeof(authority_resolution_ref),
                                                  artifact_last_ref,
                                                  sizeof(artifact_last_ref),
                                                  artifact_linkage_ref,
                                                  sizeof(artifact_linkage_ref),
                                                  NULL,
                                                  0,
                                                  NULL,
                                                  0);
    {
        int req_i;
        authority_requirements_csv[0] = '\0';
        for (req_i = 0; req_i < law_out->decision.authority_requirement_count; req_i++) {
            int wn;
            const char *tok = law_out->decision.authority_requirements[req_i];
            if (!tok || !tok[0]) continue;
            wn = snprintf(authority_requirements_csv + authority_req_used,
                          sizeof(authority_requirements_csv) - authority_req_used,
                          "%s%s",
                          authority_req_used == 0 ? "" : ",",
                          tok);
            if (wn <= 0 || (size_t)wn >= (sizeof(authority_requirements_csv) - authority_req_used)) break;
            authority_req_used += (size_t)wn;
        }
        (void)snprintf(canonical_authority_record,
                       sizeof(canonical_authority_record),
                       "{\"workspace_id\":\"%s\",\"trace_ref\":\"%s\",\"decision_ref\":\"%s\","
                       "\"authority_ref\":\"%s\",\"authority_profile\":\"%s\",\"authority_requirements\":\"%s\",\"governance_refs\":\"%s\"}",
                       ws_id,
                       law_out->evidence.trace_id,
                       decision_ref,
                       authority_last_ref,
                       law_out->decision.stack.authority_profile,
                       authority_requirements_csv,
                       info.policy_attachments_csv);
        if (yai_data_records_append_authority(ws_id,
                                              canonical_authority_record,
                                              canonical_authority_ref,
                                              sizeof(canonical_authority_ref),
                                              err,
                                              err_cap) != 0) {
            if (err && err_cap > 0 && err[0] == '\0')
                snprintf(err, err_cap, "%s", "canonical_authority_record_append_failed");
        }
        (void)snprintf(canonical_authority_resolution_record,
                       sizeof(canonical_authority_resolution_record),
                       "{\"workspace_id\":\"%s\",\"trace_ref\":\"%s\",\"decision_ref\":\"%s\","
                       "\"authority_resolution_ref\":\"%s\",\"authority_ref\":\"%s\","
                       "\"authority_context\":\"%s\",\"effect\":\"%s\",\"review_required\":%s}",
                       ws_id,
                       law_out->evidence.trace_id,
                       decision_ref,
                       authority_resolution_ref,
                       authority_last_ref,
                       law_out->evidence.authority_context,
                       yai_governance_effect_name(law_out->decision.final_effect),
                       law_out->decision.final_effect == YAI_GOVERNANCE_EFFECT_REVIEW_REQUIRED ? "true" : "false");
        if (yai_data_records_append_authority_resolution(ws_id,
                                                         canonical_authority_resolution_record,
                                                         canonical_authority_resolution_ref,
                                                         sizeof(canonical_authority_resolution_ref),
                                                         err,
                                                         err_cap) != 0) {
            if (err && err_cap > 0 && err[0] == '\0')
                snprintf(err, err_cap, "%s", "canonical_authority_resolution_record_append_failed");
        }
    }
    {
        char enforce_err[96];
        if (yai_workspace_append_enforcement_record_set(ws_id,
                                                        law_out,
                                                        info.policy_attachments_csv,
                                                        event_ref,
                                                        decision_ref,
                                                        evidence_ref,
                                                        authority_last_ref,
                                                        authority_resolution_ref,
                                                        artifact_last_ref,
                                                        artifact_linkage_ref,
                                                        enforce_err,
                                                        sizeof(enforce_err)) != 0)
        {
            if (err && err_cap > 0)
                snprintf(err, err_cap, "%s", enforce_err[0] ? enforce_err : "enforcement_record_set_failed");
            return -1;
        }
    }
    yai_workspace_read_enforcement_index(&info,
                                         enforce_last_outcome_ref,
                                         sizeof(enforce_last_outcome_ref),
                                         enforce_last_linkage_ref,
                                         sizeof(enforce_last_linkage_ref),
                                         enforce_materialization_status,
                                         sizeof(enforce_materialization_status),
                                         enforce_missing_fields,
                                         sizeof(enforce_missing_fields),
                                         NULL,
                                         0,
                                         NULL,
                                         0);
    {
        (void)snprintf(canonical_enforcement_outcome_record,
                       sizeof(canonical_enforcement_outcome_record),
                       "{\"workspace_id\":\"%s\",\"trace_ref\":\"%s\",\"decision_ref\":\"%s\","
                       "\"outcome_ref\":\"%s\",\"effect\":\"%s\",\"status\":\"%s\",\"code\":\"%s\","
                       "\"reason\":\"%s\",\"review_state\":\"%s\",\"authority_decision\":\"%s\","
                       "\"authority_constraints\":\"%s\",\"materialization_status\":\"%s\",\"missing_fields\":\"%s\"}",
                       ws_id,
                       law_out->evidence.trace_id,
                       decision_ref,
                       enforce_last_outcome_ref,
                       yai_governance_effect_name(law_out->decision.final_effect),
                       enforcement_out->status,
                       enforcement_out->code,
                       enforcement_out->reason,
                       enforcement_out->review_state,
                       enforcement_out->authority_decision == YAI_AUTHORITY_DENY ? "deny" :
                           enforcement_out->authority_decision == YAI_AUTHORITY_REVIEW_REQUIRED ? "review_required" : "allow",
                       enforcement_out->authority_constraints,
                       enforce_materialization_status,
                       enforce_missing_fields);
        if (yai_data_records_append_enforcement_outcome(ws_id,
                                                        canonical_enforcement_outcome_record,
                                                        canonical_enforcement_outcome_ref,
                                                        sizeof(canonical_enforcement_outcome_ref),
                                                        err,
                                                        err_cap) != 0 &&
            err && err_cap > 0 && err[0] == '\0') {
            snprintf(err, err_cap, "%s", "canonical_enforcement_outcome_record_append_failed");
        }
        (void)snprintf(canonical_enforcement_linkage_record,
                       sizeof(canonical_enforcement_linkage_record),
                       "{\"workspace_id\":\"%s\",\"trace_ref\":\"%s\",\"decision_ref\":\"%s\","
                       "\"linkage_ref\":\"%s\",\"event_ref\":\"%s\",\"evidence_ref\":\"%s\","
                       "\"authority_ref\":\"%s\",\"authority_resolution_ref\":\"%s\","
                       "\"artifact_ref\":\"%s\",\"artifact_linkage_ref\":\"%s\","
                       "\"governance_refs\":\"%s\",\"runtime_bound\":%s}",
                       ws_id,
                       law_out->evidence.trace_id,
                       decision_ref,
                       enforce_last_linkage_ref,
                       event_ref,
                       evidence_ref,
                       authority_last_ref,
                       authority_resolution_ref,
                       artifact_last_ref,
                       artifact_linkage_ref,
                       info.policy_attachments_csv,
                       enforcement_out->runtime_bound ? "true" : "false");
        if (yai_data_records_append_enforcement_linkage(ws_id,
                                                        canonical_enforcement_linkage_record,
                                                        canonical_enforcement_linkage_ref,
                                                        sizeof(canonical_enforcement_linkage_ref),
                                                        err,
                                                        err_cap) != 0 &&
            err && err_cap > 0 && err[0] == '\0') {
            snprintf(err, err_cap, "%s", "canonical_enforcement_linkage_record_append_failed");
        }
    }
    {
        char brain_err[96];
        int brain_rc = yai_storage_bridge_resolution_hook(ws_id,
                                                                law_out->decision.family_id,
                                                                law_out->decision.specialization_id,
                                                                yai_governance_effect_name(law_out->decision.final_effect),
                                                                law_out->decision.stack.authority_profile,
                                                                law_out->evidence.resource,
                                                                info.policy_attachments_csv,
                                                                authority_last_ref,
                                                                artifact_last_ref,
                                                                event_ref,
                                                                decision_ref,
                                                                evidence_ref,
                                                                brain_err,
                                                                sizeof(brain_err));
        if (brain_rc != YAI_MIND_OK && err && err_cap > 0 && err[0] == '\0')
            (void)snprintf(err, err_cap, "%s", brain_err[0] ? brain_err : "brain_sink_degraded");
    }

    overlays[0] = '\0';
    for (i = 0; i < stack->overlay_count; i++)
    {
        n = snprintf(overlays + used, sizeof(overlays) - used, "%s%s", (i == 0) ? "" : ",", stack->overlay_layers[i]);
        if (n <= 0 || (size_t)n >= (sizeof(overlays) - used))
            break;
        used += (size_t)n;
    }
    snprintf(info.effective_overlays_ref, sizeof(info.effective_overlays_ref), "%s", overlays);
    info.updated_at = (long)time(NULL);
    info.last_attached_at = info.updated_at;
    info.runtime_attached = 1;
    info.control_plane_attached = 1;
    if (info.session_binding[0] == '\0')
        snprintf(info.session_binding, sizeof(info.session_binding), "%s", ws_id);
    if (info.declared_context_source[0] == '\0')
        snprintf(info.declared_context_source, sizeof(info.declared_context_source), "%s", "unset");
    if (info.isolation_mode[0] == '\0')
        snprintf(info.isolation_mode, sizeof(info.isolation_mode), "%s", "process");

    if (yai_workspace_write_manifest_ws_id(ws_id, &info) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "manifest_write_failed");
        return -1;
    }
    if (yai_workspace_write_containment_surfaces(&info) != 0)
    {
        if (err && err_cap > 0)
            snprintf(err, err_cap, "%s", "containment_write_failed");
        return -1;
    }
    return 0;
}
