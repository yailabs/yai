#ifndef YAI_EFFECT_DISPATCH_ADMISSION_H
#define YAI_EFFECT_DISPATCH_ADMISSION_H

/*
 * YAI - Dispatch admission gate (CORE.ENFORCE.1)
 *
 * Implements:
 *   The deterministic, fail-closed pre-dispatch check that unifies a control
 *   decision with a CapabilityLease.
 *
 * This file owns:
 *   The rule that an executable carrier path is admitted only when a minted
 *   lease permits execution AND the control decision allows. Everything else
 *   resolves to an explicit review/no-execution or deny path.
 *
 * This file does not own:
 *   Carrier execution, decision derivation, lease minting or receipts.
 */

#include "yai/base/error.h"
#include "yai/control/capability_lease.h"
#include "yai/control/decision.h"

typedef enum yai_dispatch_admission_outcome {
    YAI_DISPATCH_ADMIT_EXECUTE = 0,
    YAI_DISPATCH_ADMIT_REVIEW = 1,
    YAI_DISPATCH_ADMIT_DENY = 2
} yai_dispatch_admission_outcome_t;

typedef struct yai_dispatch_admission {
    yai_dispatch_admission_outcome_t outcome;
    int execution_admitted; /* 1 only when outcome == YAI_DISPATCH_ADMIT_EXECUTE */
    int lease_permits;      /* result of yai_capability_lease_permits_execution */
    int decision_allows;    /* decision outcome allow / allow_with_constraints */
    char reason[128];
} yai_dispatch_admission_t;

const char *yai_dispatch_admission_outcome_to_string(
    yai_dispatch_admission_outcome_t outcome);

/*
 * Resolve whether an operation may reach an executable carrier path.
 *
 * Fail-closed: a NULL out is invalid; a NULL/non-permitting lease or a
 * non-allow decision never yields YAI_DISPATCH_ADMIT_EXECUTE. Review/defer/
 * evidence/redaction/observe decisions resolve to YAI_DISPATCH_ADMIT_REVIEW
 * (no execution). Deny and "allow without a permitting lease" resolve to
 * YAI_DISPATCH_ADMIT_DENY. execution_admitted is 1 only when both a minted
 * lease permits execution and the decision allows.
 */
yai_status_t yai_dispatch_admit(const yai_capability_lease_t *lease,
                                const yai_control_decision_t *decision,
                                yai_dispatch_admission_t *out);

#endif
