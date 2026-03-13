#include "yai/abi/errors.h"
#include "yai/kernel/grants.h"

int yai_kernel_grant_transition_allowed(
    enum yai_kernel_grant_state from,
    enum yai_kernel_grant_state to) {
    switch (from) {
        case YAI_KERNEL_GRANT_PENDING:
            return (to == YAI_KERNEL_GRANT_ACTIVE ||
                    to == YAI_KERNEL_GRANT_DENIED ||
                    to == YAI_KERNEL_GRANT_REVOKED) ? YAI_OK : YAI_ERR_DENIED;
        case YAI_KERNEL_GRANT_ACTIVE:
            return (to == YAI_KERNEL_GRANT_SUSPENDED ||
                    to == YAI_KERNEL_GRANT_REVOKED ||
                    to == YAI_KERNEL_GRANT_EXPIRED) ? YAI_OK : YAI_ERR_DENIED;
        case YAI_KERNEL_GRANT_SUSPENDED:
            return (to == YAI_KERNEL_GRANT_ACTIVE ||
                    to == YAI_KERNEL_GRANT_REVOKED ||
                    to == YAI_KERNEL_GRANT_EXPIRED) ? YAI_OK : YAI_ERR_DENIED;
        case YAI_KERNEL_GRANT_DENIED:
        case YAI_KERNEL_GRANT_REVOKED:
        case YAI_KERNEL_GRANT_EXPIRED:
            return YAI_ERR_DENIED;
        default:
            return YAI_ERR_INVALID;
    }
}

int yai_kernel_grant_set_state(struct yai_kernel_grant* grant, enum yai_kernel_grant_state state) {
    int rc;

    if (grant == 0) {
        return YAI_ERR_INVALID;
    }

    rc = yai_kernel_grant_transition_allowed(grant->validity_state, state);
    if (rc != YAI_OK) {
        return rc;
    }

    grant->validity_state = state;
    return YAI_OK;
}
