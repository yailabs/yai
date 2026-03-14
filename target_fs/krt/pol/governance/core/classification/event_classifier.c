#include "../internal.h"

#include <string.h>

int yai_governance_classify_event(const char *ws_id,
                           const char *payload,
                           yai_governance_classification_ctx_t *out) {
  if (!out || !payload) return -1;
  memset(out, 0, sizeof(*out));

  if (ws_id && ws_id[0]) {
    (void)yai_governance_safe_snprintf(out->ws_id, sizeof(out->ws_id), "%s", ws_id);
  }

  if (yai_governance_classify_action(payload, out->action, sizeof(out->action)) != 0) return -1;
  if (yai_governance_classify_provider(payload, out->provider, sizeof(out->provider)) != 0) return -1;
  if (yai_governance_classify_resource(payload, out->resource, sizeof(out->resource)) != 0) return -1;
  if (yai_governance_classify_protocol(payload, out->protocol, sizeof(out->protocol)) != 0) return -1;
  (void)yai_governance_json_extract_string(payload, "workspace_declared_family", out->declared_family, sizeof(out->declared_family));
  (void)yai_governance_json_extract_string(payload, "workspace_declared_specialization", out->declared_specialization, sizeof(out->declared_specialization));
  if (yai_governance_extract_workspace_context(payload,
                                        out->workspace_mode,
                                        sizeof(out->workspace_mode),
                                        &out->black_box_mode,
                                        &out->has_params_hash,
                                        &out->has_authority_contract,
                                        &out->has_repro_context,
                                        &out->has_dataset_ref,
                                        &out->has_publication_intent,
                                        &out->has_locked_parameters,
                                        &out->has_result_ref,
                                        &out->has_retrieve_intent,
                                        &out->has_egress_intent,
                                        &out->has_commentary_intent,
                                        &out->has_distribution_intent,
                                        &out->has_sink_ref,
                                        &out->sink_trusted,
                                        &out->sink_external) != 0) {
    return -1;
  }

  if (yai_governance_json_extract_string(payload, "command", out->command, sizeof(out->command)) != 0) {
    if (strstr(payload, "curl")) (void)yai_governance_safe_snprintf(out->command, sizeof(out->command), "%s", "curl");
    else if (strstr(payload, "otel")) (void)yai_governance_safe_snprintf(out->command, sizeof(out->command), "%s", "otel.export");
    else if (strstr(payload, "s3")) (void)yai_governance_safe_snprintf(out->command, sizeof(out->command), "%s", "s3.put_object");
    else if (strstr(payload, "payment")) (void)yai_governance_safe_snprintf(out->command, sizeof(out->command), "%s", "payment.authorize");
    else if (strstr(payload, "transfer")) (void)yai_governance_safe_snprintf(out->command, sizeof(out->command), "%s", "transfer.authorize");
    else if (strstr(payload, "settlement")) (void)yai_governance_safe_snprintf(out->command, sizeof(out->command), "%s", "settlement.finalize");
    else if (strstr(payload, "github")) (void)yai_governance_safe_snprintf(out->command, sizeof(out->command), "%s", "github.issues.comment.create");
    else if (strstr(payload, "distribution") || strstr(payload, "deliver")) (void)yai_governance_safe_snprintf(out->command, sizeof(out->command), "%s", "artifact.distribution.push");
    else if (strstr(payload, "sink") || strstr(payload, "destination")) (void)yai_governance_safe_snprintf(out->command, sizeof(out->command), "%s", "digital.sink.validate");
    else if (strstr(payload, "retrieve") || strstr(payload, "fetch")) (void)yai_governance_safe_snprintf(out->command, sizeof(out->command), "%s", "remote.retrieve");
    else if (strstr(payload, "experiment")) (void)yai_governance_safe_snprintf(out->command, sizeof(out->command), "%s", "experiment.run");
    else (void)yai_governance_safe_snprintf(out->command, sizeof(out->command), "%s", "unknown");
  }

  return 0;
}
