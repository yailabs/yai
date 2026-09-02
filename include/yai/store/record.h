#ifndef YAI_STORE_RECORD_H
#define YAI_STORE_RECORD_H

/*
 * Research traceability:
 * YAI-RI-I1 - Ordered history / filtration preservation.
 * See docs/operational-state-mathematics.md#yai-ri-i1-ordered-history
 *
 * YAI-RI-I2 - State provenance.
 * See docs/operational-state-mathematics.md#yai-ri-i2-state-provenance
 */

#include "yai/base/error.h"
#include "yai/base/id.h"
#include "yai/case/case_ref.h"
#include "yai/subject/subject_ref.h"

typedef enum yai_record_kind {
    YAI_RECORD_CASE = 0,
    YAI_RECORD_SUBJECT_BINDING = 1,
    YAI_RECORD_ATTEMPT = 2,
    YAI_RECORD_DECISION = 3,
    YAI_RECORD_RECEIPT = 4,
    YAI_RECORD_PROJECTION = 5,
    YAI_RECORD_POLICY_RULE = 6,
    YAI_RECORD_GATE_RESULT = 7,
    YAI_RECORD_DECISION_BASIS = 8,
    YAI_RECORD_OBLIGATION = 9,
    YAI_RECORD_RECEIPT_REQUIREMENT = 10,
    YAI_RECORD_CARRIER_REQUEST = 11,
    YAI_RECORD_EFFECT_RECEIPT = 12,
    YAI_RECORD_FILESYSTEM_RECEIPT = 13,
    YAI_RECORD_SUBJECT_STATE = 14,
    YAI_RECORD_GRAPH_EDGE = 15,
    YAI_RECORD_RECONSTRUCTION = 16,
    YAI_RECORD_MEMORY_CANDIDATE = 17,
    YAI_RECORD_DIVERGENCE = 18,
    YAI_RECORD_RECONCILIATION = 19,
    YAI_RECORD_PROJECTION_REQUEST = 20,
    YAI_RECORD_PROJECTION_RESULT = 21,
    YAI_RECORD_QUERY_RESULT = 22,
    YAI_RECORD_PROJECTION_RULE = 23,
    YAI_RECORD_AUTHORITY_SCOPE = 24,
    YAI_RECORD_MODEL_INTERPRETATION = 25,
    YAI_RECORD_CASE_DOMAIN = 26,
    YAI_RECORD_CASE_ATTACHMENT = 27,
    YAI_RECORD_CASE_BINDING = 28,
    YAI_RECORD_INTERACTION_THREAD = 29,
    YAI_RECORD_INTERACTION_TURN = 30,
    YAI_RECORD_PARTICIPANT_VIEW_FRAME = 31
} yai_record_kind_t;

typedef struct yai_store_record {
    yai_id_t record_id;
    yai_case_ref_t case_ref;
    yai_record_kind_t record_kind;
    yai_subject_ref_t subject_ref;
    yai_id_t attempt_id;
    yai_id_t decision_id;
    yai_id_t receipt_id;
    char summary[128];
} yai_store_record_t;

yai_status_t yai_store_record_init(yai_store_record_t *record,
                                   const char *record_id,
                                   const yai_case_ref_t *case_ref,
                                   yai_record_kind_t record_kind,
                                   const yai_subject_ref_t *subject_ref,
                                   const yai_id_t *attempt_id,
                                   const yai_id_t *decision_id,
                                   const yai_id_t *receipt_id,
                                   const char *summary);
const char *yai_record_kind_string(yai_record_kind_t kind);
yai_status_t yai_record_kind_from_string(const char *value,
                                         yai_record_kind_t *kind);

#endif
