/*
 * YAI - Dispatch admission gate (CORE.ENFORCE.1)
 *
 * Implements:
 *   yai_dispatch_admit: the deterministic, fail-closed pre-dispatch check that
 *   requires a permitting CapabilityLease AND an allow decision before any
 *   executable carrier path may run.
 *
 * This file owns:
 *   The lease+decision admission rule and its outcome vocabulary.
 *
 * This file does not own:
 *   Carrier execution, decision derivation, lease minting or receipts.
 */

#include "yai/effect/dispatch_admission.h"

#include <string.h>

void yai_copy_string(char *dst, size_t dst_size, const char *src);

const char *yai_dispatch_admission_outcome_to_string(
    yai_dispatch_admission_outcome_t outcome) {
    switch (outcome) {
    case YAI_DISPATCH_ADMIT_EXECUTE:
        return "execute";
    case YAI_DISPATCH_ADMIT_REVIEW:
        return "review";
    case YAI_DISPATCH_ADMIT_DENY:
        return "deny";
    default:
        return "deny";
    }
}

yai_status_t yai_dispatch_admit(const yai_capability_lease_t *lease,
                                const yai_control_decision_t *decision,
                                yai_dispatch_admission_t *out) {
    if (out == 0) {
        return YAI_ERR_INVALID;
    }

    /* Fail-closed default: no execution until both checks pass. */
    out->outcome = YAI_DISPATCH_ADMIT_DENY;
    out->execution_admitted = 0;
    out->lease_permits = 0;
    out->decision_allows = 0;
    yai_copy_string(out->reason, sizeof(out->reason), "fail_closed_default");

    if (decision == 0) {
        yai_copy_string(out->reason, sizeof(out->reason), "missing_decision");
        return YAI_OK;
    }

    out->lease_permits = yai_capability_lease_permits_execution(lease);
    out->decision_allows =
        (decision->outcome == YAI_DECISION_ALLOW ||
         decision->outcome == YAI_DECISION_ALLOW_WITH_CONSTRAINTS);

    /* Explicit no-execution review path. */
    if (decision->outcome == YAI_DECISION_REQUIRE_REVIEW ||
        decision->outcome == YAI_DECISION_DEFER ||
        decision->outcome == YAI_DECISION_REQUIRE_EVIDENCE ||
        decision->outcome == YAI_DECISION_REQUIRE_REDACTION ||
        decision->outcome == YAI_DECISION_OBSERVE_ONLY) {
        out->outcome = YAI_DISPATCH_ADMIT_REVIEW;
        yai_copy_string(out->reason, sizeof(out->reason), "review_no_execution");
        return YAI_OK;
    }

    if (decision->outcome == YAI_DECISION_DENY) {
        out->outcome = YAI_DISPATCH_ADMIT_DENY;
        yai_copy_string(out->reason, sizeof(out->reason), "decision_denied");
        return YAI_OK;
    }

    /* An allow decision is necessary but not sufficient: a minted lease must
     * also permit execution. This is the CORE.ENFORCE.1 boundary. */
    if (out->decision_allows && out->lease_permits) {
        out->outcome = YAI_DISPATCH_ADMIT_EXECUTE;
        out->execution_admitted = 1;
        yai_copy_string(out->reason, sizeof(out->reason),
                        "lease_and_decision_admit_execution");
        return YAI_OK;
    }

    if (out->decision_allows && !out->lease_permits) {
        out->outcome = YAI_DISPATCH_ADMIT_DENY;
        yai_copy_string(out->reason, sizeof(out->reason),
                        "lease_does_not_permit_execution");
        return YAI_OK;
    }

    yai_copy_string(out->reason, sizeof(out->reason), "no_execution_authority");
    return YAI_OK;
}
