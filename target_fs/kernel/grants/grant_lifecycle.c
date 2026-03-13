#include <string.h>

#include "yai/abi/errors.h"
#include "yai/kernel/grants.h"
#include "yai/kernel/lifecycle.h"
#include "yai/kernel/registry.h"

int yai_kernel_grant_set_state(struct yai_kernel_grant* grant, enum yai_kernel_grant_state state);

int yai_kernel_grant_issue(const struct yai_kernel_grant_request* request, struct yai_kernel_grant* out_grant) {
    struct yai_kernel_grant grant;
    int rc;

    if (request == 0 || out_grant == 0 || request->grant_id == 0 || request->subject_handle == 0) {
        return YAI_ERR_INVALID;
    }

    rc = yai_kernel_can_issue_grants();
    if (rc != YAI_OK) {
        return rc;
    }

    memset(&grant, 0, sizeof(grant));
    grant.grant_id = request->grant_id;
    grant.subject_handle = request->subject_handle;
    grant.capability_class = request->capability_class;
    grant.scope_handle = request->scope_handle;
    grant.validity_state = YAI_KERNEL_GRANT_PENDING;
    grant.issued_at = request->issued_at;
    grant.expires_at = request->expires_at;
    grant.revoked_at = 0;
    grant.flags = request->flags;

    rc = yai_grants_registry_set(&grant);
    if (rc != YAI_OK) {
        return rc;
    }

    *out_grant = grant;
    return YAI_OK;
}

int yai_kernel_grant_activate(yai_object_id_t grant_id) {
    struct yai_kernel_grant grant;
    int rc;

    rc = yai_grants_registry_get(grant_id, &grant);
    if (rc != YAI_OK) {
        return rc;
    }

    rc = yai_kernel_grant_set_state(&grant, YAI_KERNEL_GRANT_ACTIVE);
    if (rc != YAI_OK) {
        return rc;
    }

    return yai_grants_registry_set(&grant);
}

int yai_kernel_grant_suspend(yai_object_id_t grant_id, uint64_t flags) {
    struct yai_kernel_grant grant;
    int rc;

    rc = yai_grants_registry_get(grant_id, &grant);
    if (rc != YAI_OK) {
        return rc;
    }

    rc = yai_kernel_grant_set_state(&grant, YAI_KERNEL_GRANT_SUSPENDED);
    if (rc != YAI_OK) {
        return rc;
    }

    grant.flags |= flags;
    return yai_grants_registry_set(&grant);
}

int yai_kernel_grant_revoke(yai_object_id_t grant_id, uint64_t revoked_at, uint64_t flags) {
    struct yai_kernel_grant grant;
    int rc;

    rc = yai_grants_registry_get(grant_id, &grant);
    if (rc != YAI_OK) {
        return rc;
    }

    rc = yai_kernel_grant_set_state(&grant, YAI_KERNEL_GRANT_REVOKED);
    if (rc != YAI_OK) {
        return rc;
    }

    grant.revoked_at = revoked_at;
    grant.flags |= flags;
    return yai_grants_registry_set(&grant);
}

int yai_kernel_grant_expire(yai_object_id_t grant_id, uint64_t expired_at) {
    struct yai_kernel_grant grant;
    int rc;

    rc = yai_grants_registry_get(grant_id, &grant);
    if (rc != YAI_OK) {
        return rc;
    }

    rc = yai_kernel_grant_set_state(&grant, YAI_KERNEL_GRANT_EXPIRED);
    if (rc != YAI_OK) {
        return rc;
    }

    grant.expires_at = expired_at;
    return yai_grants_registry_set(&grant);
}
