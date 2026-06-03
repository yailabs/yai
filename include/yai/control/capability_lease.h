#ifndef YAI_CONTROL_CAPABILITY_LEASE_H
#define YAI_CONTROL_CAPABILITY_LEASE_H

#include <stdint.h>

#include "yai/base/error.h"
#include "yai/case/case_ref.h"
#include "yai/subject/subject_ref.h"

typedef struct yai_capability_lease {
    char capability_lease_id[128];
    yai_case_ref_t case_ref;
    yai_subject_ref_t subject_ref;
    char operation_family[64];
    char carrier_family[64];
    char resource_scope_id[128];
    char lease_status[64];
    char allowed_actions[128];
    char constraints[256];
    int requires_review;
    int receipt_required;
    uint64_t expires_at_unix_ms;
    char policy_basis[128];
    char authority_basis[128];
    char visibility_basis[128];
} yai_capability_lease_t;

yai_status_t yai_capability_lease_init(yai_capability_lease_t *lease,
                                       const yai_case_ref_t *case_ref,
                                       const yai_subject_ref_t *subject_ref,
                                       const char *operation_family);

/*
 * Fail-closed execution predicate.
 *
 * Returns 1 only when the lease is a minted, non-review lease that grants at
 * least one concrete action. A NULL lease, a default/not_applicable lease, a
 * review-bound lease, or a lease whose allowed_actions is empty/"none" returns
 * 0. A ref, binding or proposal that did not mint a lease therefore cannot
 * authorize execution.
 */
int yai_capability_lease_permits_execution(const yai_capability_lease_t *lease);

#endif
