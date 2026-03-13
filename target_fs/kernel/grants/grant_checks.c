#include "yai/abi/errors.h"
#include "yai/kernel/grants.h"
#include "yai/kernel/registry.h"

int yai_kernel_grant_check(
    yai_object_id_t subject_handle,
    enum yai_kernel_capability_class capability_class,
    yai_object_id_t scope_handle,
    struct yai_kernel_grant* out_grant) {
    int rc;

    if (out_grant == 0) {
        return YAI_ERR_INVALID;
    }

    rc = yai_grants_registry_find_active(subject_handle, capability_class, scope_handle, out_grant);
    if (rc != YAI_OK) {
        return rc;
    }

    if (out_grant->validity_state != YAI_KERNEL_GRANT_ACTIVE) {
        return YAI_ERR_DENIED;
    }

    return YAI_OK;
}
