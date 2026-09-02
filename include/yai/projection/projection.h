#ifndef YAI_PROJECTION_PROJECTION_H
#define YAI_PROJECTION_PROJECTION_H

/*
 * Research traceability:
 * YAI-RI-I7 - Context inclusion and omission disclosure.
 * See docs/operational-state-mathematics.md#yai-ri-i7-context-omission-disclosure
 */

#include <stddef.h>

#include "yai/base/error.h"
#include "yai/base/id.h"
#include "yai/case/case_ref.h"
#include "yai/store/journal.h"

typedef enum yai_projection_consumer {
    YAI_PROJECTION_OPERATOR = 0,
    YAI_PROJECTION_MODEL = 1,
    YAI_PROJECTION_AGENT = 2,
    YAI_PROJECTION_API = 3,
    YAI_PROJECTION_AUDIT = 4,
    YAI_PROJECTION_DEBUG = 5
} yai_projection_consumer_t;

typedef struct yai_projection {
    yai_id_t projection_id;
    yai_case_ref_t case_ref;
    yai_projection_consumer_t consumer_kind;
    char summary[512];
    size_t source_record_count;
    size_t decision_count;
    size_t policy_rule_count;
    size_t gate_count;
    size_t obligation_count;
    size_t receipt_requirement_count;
    size_t filesystem_receipt_count;
    size_t subject_state_count;
    size_t effect_count;
    size_t graph_edge_count;
    size_t reconstruction_count;
    size_t memory_candidate_count;
    size_t operational_memory_candidate_count;
    size_t decision_memory_candidate_count;
    size_t subject_memory_candidate_count;
    size_t error_memory_candidate_count;
    size_t recovery_memory_candidate_count;
    size_t divergence_count;
    size_t reconciliation_count;
    size_t critical_divergence_count;
    size_t warning_divergence_count;
    size_t projection_request_count;
    size_t projection_result_count;
    size_t operator_projection_count;
    size_t model_projection_count;
    size_t audit_projection_count;
    size_t limited_projection_count;
    size_t query_result_count;
    size_t projection_rule_count;
    size_t authority_scope_count;
    size_t model_interpretation_count;
    size_t case_domain_count;
    size_t case_attachment_count;
    size_t case_binding_count;
    size_t interaction_thread_count;
    size_t interaction_turn_count;
    size_t participant_view_frame_count;
} yai_projection_t;

yai_status_t yai_projection_build(const char *projection_id,
                                  const yai_case_ref_t *case_ref,
                                  yai_projection_consumer_t consumer_kind,
                                  const yai_journal_t *journal,
                                  yai_projection_t *projection);
const char *yai_projection_consumer_string(yai_projection_consumer_t consumer);

#endif
