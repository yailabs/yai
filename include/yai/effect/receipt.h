#ifndef YAI_EFFECT_RECEIPT_H
#define YAI_EFFECT_RECEIPT_H

/*
 * Research traceability:
 * YAI-RI-I5 - Receipt coupling for state-changing effects.
 * See docs/operational-state-mathematics.md#yai-ri-i5-receipt-coupling
 */

#include <stddef.h>

#include "yai/base/error.h"
#include "yai/base/id.h"
#include "yai/case/case_ref.h"
#include "yai/control/decision.h"
#include "yai/effect/carrier.h"
#include "yai/op/attempt.h"
#include "yai/subject/subject_ref.h"

typedef enum yai_receipt_status {
    YAI_RECEIPT_EXECUTED = 0,
    YAI_RECEIPT_OBSERVED = 1,
    YAI_RECEIPT_BLOCKED = 2,
    YAI_RECEIPT_FAILED = 3,
    YAI_RECEIPT_IMPORTED = 4,
    YAI_RECEIPT_UNKNOWN = 5
} yai_receipt_status_t;

typedef struct yai_effect_receipt {
    yai_id_t receipt_id;
    yai_case_ref_t case_ref;
    yai_id_t attempt_id;
    yai_id_t decision_id;
    yai_subject_ref_t target_subject_ref;
    yai_receipt_status_t status;
    yai_carrier_kind_t carrier_kind;
    yai_effect_kind_t effect_kind;
    char path_or_locator[128];
    size_t bytes_affected;
    char before_hash[32];
    char after_hash[32];
    char summary[128];
} yai_effect_receipt_t;

yai_status_t yai_effect_receipt_from_decision(const char *receipt_id,
                                              const yai_op_attempt_t *attempt,
                                              const yai_control_decision_t *decision,
                                              yai_effect_receipt_t *receipt);
const char *yai_receipt_status_string(yai_receipt_status_t status);

#endif
