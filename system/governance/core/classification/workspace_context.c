#include "../internal.h"

#include <string.h>

int yai_governance_extract_workspace_context(const char *payload,
                                      char *out_mode,
                                      size_t out_mode_cap,
                                      int *black_box_mode,
                                      int *has_params_hash,
                                      int *has_authority_contract,
                                      int *has_repro_context,
                                      int *has_dataset_ref,
                                      int *has_publication_intent,
                                      int *has_locked_parameters,
                                      int *has_result_ref,
                                      int *has_retrieve_intent,
                                      int *has_egress_intent,
                                      int *has_commentary_intent,
                                      int *has_distribution_intent,
                                      int *has_sink_ref,
                                      int *sink_trusted,
                                      int *sink_external) {
  if (!payload || !out_mode || !black_box_mode || !has_params_hash || !has_authority_contract ||
      !has_repro_context || !has_dataset_ref || !has_publication_intent || !has_locked_parameters ||
      !has_result_ref || !has_retrieve_intent || !has_egress_intent || !has_commentary_intent ||
      !has_distribution_intent || !has_sink_ref || !sink_trusted || !sink_external) return -1;

  *black_box_mode = (strstr(payload, "black-box") || strstr(payload, "black_box")) ? 1 : 0;
  *has_params_hash = (strstr(payload, "params_hash") || strstr(payload, "param-lock")) ? 1 : 0;
  *has_authority_contract = (strstr(payload, "allowlist") || strstr(payload, "contract") || strstr(payload, "approved")) ? 1 : 0;
  *has_repro_context = (strstr(payload, "reproduc") || strstr(payload, "lineage") || strstr(payload, "provenance") ||
                        strstr(payload, "seed_hash") || strstr(payload, "proofpack")) ? 1 : 0;
  *has_dataset_ref = (strstr(payload, "dataset") || strstr(payload, "corpus") || strstr(payload, "input_set")) ? 1 : 0;
  *has_publication_intent = (strstr(payload, "publish") || strstr(payload, "export") || strstr(payload, "release")) ? 1 : 0;
  *has_locked_parameters = (strstr(payload, "param-lock") || strstr(payload, "locked_param") || strstr(payload, "immutable_param")) ? 1 : 0;
  *has_result_ref = (strstr(payload, "result") || strstr(payload, "artifact") || strstr(payload, "evaluation_output")) ? 1 : 0;
  *has_retrieve_intent = (strstr(payload, "retrieve") || strstr(payload, "fetch") || strstr(payload, "pull")) ? 1 : 0;
  *has_egress_intent = (strstr(payload, "egress") || strstr(payload, "outbound") || strstr(payload, "send") ||
                        strstr(payload, "push") || strstr(payload, "upload")) ? 1 : 0;
  *has_commentary_intent = (strstr(payload, "comment") || strstr(payload, "annotation") ||
                            strstr(payload, "statement") || strstr(payload, "reply")) ? 1 : 0;
  *has_distribution_intent = (strstr(payload, "distribut") || strstr(payload, "broadcast") ||
                              strstr(payload, "share") || strstr(payload, "deliver")) ? 1 : 0;
  *has_sink_ref = (strstr(payload, "sink") || strstr(payload, "destination") || strstr(payload, "target") ||
                   strstr(payload, "channel")) ? 1 : 0;
  *sink_trusted = (strstr(payload, "trusted") || strstr(payload, "internal") || strstr(payload, "approved_sink")) ? 1 : 0;
  *sink_external = (strstr(payload, "external") || strstr(payload, "public") || strstr(payload, "third_party")) ? 1 : 0;

  if (*black_box_mode) return yai_governance_safe_snprintf(out_mode, out_mode_cap, "%s", "black_box");
  return yai_governance_safe_snprintf(out_mode, out_mode_cap, "%s", "default");
}
