#ifndef YAI_CONTROL_GATE_H
#define YAI_CONTROL_GATE_H

/*
 * Research traceability:
 * YAI-RI-I4 - Gated side effects.
 * See docs/operational-state-mathematics.md#yai-ri-i4-gated-side-effects
 */

#include "yai/base/error.h"
#include "yai/base/id.h"
#include "yai/case/case_ref.h"
#include "yai/control/policy_rule.h"
#include "yai/op/attempt.h"

typedef enum yai_gate_kind {
    YAI_GATE_CASE = 0,
    YAI_GATE_SUBJECT = 1,
    YAI_GATE_ACTOR = 2,
    YAI_GATE_OPERATION = 3,
    YAI_GATE_CARRIER = 4,
    YAI_GATE_PROJECTION = 5,
    YAI_GATE_RETENTION = 6
} yai_gate_kind_t;

typedef enum yai_gate_result_kind {
    YAI_GATE_PASS = 0,
    YAI_GATE_BLOCK = 1,
    YAI_GATE_REQUIRE_REVIEW = 2,
    YAI_GATE_REQUIRE_EVIDENCE = 3,
    YAI_GATE_NOT_APPLICABLE = 4,
    YAI_GATE_DEFER = 5
} yai_gate_result_kind_t;

typedef struct yai_gate_result {
    yai_id_t gate_id;
    yai_case_ref_t case_ref;
    yai_id_t attempt_id;
    yai_id_t rule_id;
    yai_gate_kind_t gate_kind;
    yai_gate_result_kind_t result;
    char reason[128];
} yai_gate_result_t;

yai_status_t yai_gate_evaluate_attempt(const yai_op_attempt_t *attempt,
                                       const yai_policy_rule_t *rule,
                                       yai_gate_result_t *gate);
const char *yai_gate_kind_string(yai_gate_kind_t kind);
const char *yai_gate_result_string(yai_gate_result_kind_t result);

#endif
