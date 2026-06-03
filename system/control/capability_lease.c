#include "yai/control/capability_lease.h"

#include <stdio.h>
#include <string.h>

void yai_copy_string(char *dst, size_t dst_size, const char *src);

yai_status_t yai_capability_lease_init(yai_capability_lease_t *lease,
                                       const yai_case_ref_t *case_ref,
                                       const yai_subject_ref_t *subject_ref,
                                       const char *operation_family) {
    if (lease == 0 || yai_case_ref_is_empty(case_ref) || yai_subject_ref_is_empty(subject_ref) ||
        operation_family == 0) {
        return YAI_ERR_INVALID;
    }
    memset(lease, 0, sizeof(*lease));
    lease->case_ref = *case_ref;
    lease->subject_ref = *subject_ref;
    (void)snprintf(lease->capability_lease_id,
                   sizeof(lease->capability_lease_id),
                   "capability-lease:%.48s:%.48s",
                   case_ref->case_id.value,
                   subject_ref->subject_id.value);
    yai_copy_string(lease->operation_family, sizeof(lease->operation_family), operation_family);
    yai_copy_string(lease->carrier_family, sizeof(lease->carrier_family), "filesystem");
    yai_copy_string(lease->lease_status, sizeof(lease->lease_status), "not_applicable");
    yai_copy_string(lease->allowed_actions, sizeof(lease->allowed_actions), "none");
    yai_copy_string(lease->constraints, sizeof(lease->constraints), "intersection_subject_authority_resource_policy_generation");
    yai_copy_string(lease->policy_basis, sizeof(lease->policy_basis), "derived_v0");
    yai_copy_string(lease->authority_basis, sizeof(lease->authority_basis), "authority-scope:derived-v0");
    yai_copy_string(lease->visibility_basis, sizeof(lease->visibility_basis), "visibility-scope:derived-v0");
    return YAI_OK;
}

int yai_capability_lease_permits_execution(const yai_capability_lease_t *lease) {
    if (lease == 0) {
        return 0;
    }
    /* A review-bound lease never directly authorizes execution. */
    if (lease->requires_review) {
        return 0;
    }
    /* Only a minted lease grants execution; the default init posture is
     * "not_applicable" / allowed_actions "none", which fails closed. */
    if (strcmp(lease->lease_status, "minted") != 0) {
        return 0;
    }
    if (lease->allowed_actions[0] == '\0' ||
        strcmp(lease->allowed_actions, "none") == 0) {
        return 0;
    }
    return 1;
}
