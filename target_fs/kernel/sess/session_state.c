#include "yai/abi/errors.h"
#include "internal.h"

int yai_kernel_session_suspend(yai_object_id_t session_id, uint64_t flags) {
    struct yai_kernel_session session;
    int rc;

    if (session_id == 0) {
        return YAI_ERR_INVALID;
    }

    rc = yai_kernel_session_load(session_id, &session);
    if (rc != YAI_OK) {
        return rc;
    }

    if (session.admission_state != YAI_KERNEL_SESSION_ADMITTED &&
        session.admission_state != YAI_KERNEL_SESSION_BOUND) {
        return YAI_ERR_DENIED;
    }

    session.admission_state = YAI_KERNEL_SESSION_SUSPENDED;
    session.flags |= flags;
    return yai_kernel_session_store(&session);
}

int yai_kernel_session_revoke(yai_object_id_t session_id, uint64_t revoked_at, uint64_t flags) {
    struct yai_kernel_session session;
    int rc;

    if (session_id == 0) {
        return YAI_ERR_INVALID;
    }

    rc = yai_kernel_session_load(session_id, &session);
    if (rc != YAI_OK) {
        return rc;
    }

    if (session.admission_state == YAI_KERNEL_SESSION_CLOSED) {
        return YAI_ERR_DENIED;
    }

    session.admission_state = YAI_KERNEL_SESSION_REVOKED;
    session.revoked_at = revoked_at;
    session.flags |= flags;
    return yai_kernel_session_store(&session);
}

int yai_kernel_session_close(yai_object_id_t session_id) {
    struct yai_kernel_session session;
    int rc;

    if (session_id == 0) {
        return YAI_ERR_INVALID;
    }

    rc = yai_kernel_session_load(session_id, &session);
    if (rc != YAI_OK) {
        return rc;
    }

    session.admission_state = YAI_KERNEL_SESSION_CLOSED;
    return yai_kernel_session_store(&session);
}
