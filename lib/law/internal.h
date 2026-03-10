#pragma once

#include <stddef.h>

#include <yai/law/classifier.h>
#include <yai/law/decision_map.h>
#include <yai/law/discovery.h>
#include <yai/law/loader.h>

int yai_law_read_text_file(const char *path, char *out, size_t out_cap);
int yai_law_json_extract_string(const char *json, const char *key, char *out, size_t out_cap);
int yai_law_json_contains(const char *json, const char *needle);
int yai_law_safe_snprintf(char *out, size_t out_cap, const char *fmt, ...);

int yai_law_classify_action(const char *payload, char *out, size_t out_cap);
int yai_law_classify_provider(const char *payload, char *out, size_t out_cap);
int yai_law_classify_resource(const char *payload, char *out, size_t out_cap);
int yai_law_classify_protocol(const char *payload, char *out, size_t out_cap);
int yai_law_extract_workspace_context(const char *payload, char *out_mode, size_t out_mode_cap,
                                      int *black_box_mode, int *has_params_hash, int *has_authority_contract,
                                      int *has_repro_context, int *has_dataset_ref, int *has_publication_intent,
                                      int *has_locked_parameters, int *has_result_ref,
                                      int *has_retrieve_intent, int *has_egress_intent,
                                      int *has_commentary_intent, int *has_distribution_intent,
                                      int *has_sink_ref, int *sink_trusted, int *sink_external);

int yai_law_match_signal_score(const yai_law_classification_ctx_t *ctx,
                               const char *domain_id,
                               double *score,
                               char *rationale,
                               size_t rationale_cap);

int yai_law_stack_build(const yai_law_runtime_t *rt,
                        const yai_law_discovery_result_t *discovery,
                        const yai_law_classification_ctx_t *ctx,
                        yai_law_effective_stack_t *stack,
                        yai_law_effect_t *effect,
                        char *rationale,
                        size_t rationale_cap);

int yai_law_domain_merge_apply(yai_law_discovery_result_t *discovery,
                               char *err,
                               size_t err_cap);
int yai_law_foundation_merge_apply(yai_law_effective_stack_t *stack,
                                   yai_law_effect_t *effect);
int yai_law_compliance_merge_apply(yai_law_effective_stack_t *stack,
                                   yai_law_effect_t *effect);
int yai_law_overlay_merge_apply(yai_law_effective_stack_t *stack);

int yai_law_build_trace_json(const yai_law_classification_ctx_t *ctx,
                             const yai_law_discovery_result_t *disc,
                             const yai_law_decision_t *decision,
                             char *out,
                             size_t out_cap);

int yai_law_overlay_loader_validate(const yai_law_runtime_t *rt, char *err, size_t err_cap);
