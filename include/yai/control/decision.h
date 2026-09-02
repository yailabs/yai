#ifndef YAI_CONTROL_DECISION_H
#define YAI_CONTROL_DECISION_H

/*
 * Research traceability:
 * YAI-RI-I4 - Gated side effects.
 * See docs/operational-state-mathematics.md#yai-ri-i4-gated-side-effects
 */

#include "yai/base/error.h"
#include "yai/base/id.h"
#include "yai/case/case_ref.h"
#include "yai/control/decision_basis.h"
#include "yai/control/gate.h"
#include "yai/op/attempt.h"

typedef enum yai_decision_outcome {
    YAI_DECISION_ALLOW = 0,
    YAI_DECISION_DENY = 1,
    YAI_DECISION_DEFER = 2,
    YAI_DECISION_OBSERVE_ONLY = 3,
    YAI_DECISION_REQUIRE_REVIEW = 4,
    YAI_DECISION_REQUIRE_EVIDENCE = 5,
    YAI_DECISION_REQUIRE_REDACTION = 6,
    YAI_DECISION_ALLOW_WITH_CONSTRAINTS = 7
} yai_decision_outcome_t;

typedef struct yai_control_decision {
    yai_id_t decision_id;
    yai_case_ref_t case_ref;
    yai_id_t attempt_id;
    yai_id_t basis_id;
    yai_id_t gate_id;
    yai_id_t rule_id;
    yai_id_t obligation_id;
    yai_id_t receipt_requirement_id;
    yai_decision_outcome_t outcome;
    char reason[128];
    int receipt_required;
} yai_control_decision_t;

yai_status_t yai_control_decide(const yai_op_attempt_t *attempt,
                                yai_control_decision_t *decision);
yai_status_t yai_control_decide_from_gate(const yai_gate_result_t *gate,
                                          const yai_decision_basis_t *basis,
                                          yai_control_decision_t *decision);
const char *yai_decision_outcome_string(yai_decision_outcome_t outcome);

#endif
