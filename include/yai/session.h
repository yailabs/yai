#pragma once

#ifndef YAI_KERNEL_SESSION_H
#define YAI_KERNEL_SESSION_H

#include <stdint.h>
#include "objects.h"

/* Canonical kernel-owned session classes. */
enum yai_kernel_session_type {
    YAI_KERNEL_SESSION_GLOBAL = 0,
    YAI_KERNEL_SESSION_CONTAINER_BOUND = 1,
    YAI_KERNEL_SESSION_PRIVILEGED = 2,
    YAI_KERNEL_SESSION_SERVICE = 3
};

/* Canonical kernel-owned privilege classes. */
enum yai_kernel_privilege_class {
    YAI_KERNEL_PRIV_USER = 0,
    YAI_KERNEL_PRIV_OPERATOR = 1,
    YAI_KERNEL_PRIV_CONTROLLER = 2,
    YAI_KERNEL_PRIV_SYSTEM = 3,
    YAI_KERNEL_PRIV_PRIVILEGED = 4
};

/* Canonical admission states controlled by the kernel. */
enum yai_kernel_session_admission_state {
    YAI_KERNEL_SESSION_PENDING = 0,
    YAI_KERNEL_SESSION_ADMITTED = 1,
    YAI_KERNEL_SESSION_BOUND = 2,
    YAI_KERNEL_SESSION_SUSPENDED = 3,
    YAI_KERNEL_SESSION_REVOKED = 4,
    YAI_KERNEL_SESSION_CLOSED = 5
};

struct yai_kernel_session {
    yai_object_id_t session_id;
    enum yai_kernel_session_type session_type;
    enum yai_kernel_privilege_class privilege_class;
    uint32_t admission_source;
    enum yai_kernel_session_admission_state admission_state;
    yai_object_id_t bound_container_id;
    uint64_t capability_mask;
    uint64_t created_at;
    uint64_t revoked_at;
    uint64_t flags;
};

struct yai_kernel_session_request {
    enum yai_kernel_session_type requested_type;
    enum yai_kernel_privilege_class requested_privilege_class;
    uint32_t admission_source;
    uint64_t capability_mask;
    uint64_t created_at;
    uint64_t flags;
};

int yai_kernel_session_admit(
    yai_object_id_t session_id,
    const struct yai_kernel_session_request* request,
    struct yai_kernel_session* out_session);

int yai_kernel_session_bind_container(yai_object_id_t session_id, yai_object_id_t container_id);
int yai_kernel_session_suspend(yai_object_id_t session_id, uint64_t flags);
int yai_kernel_session_revoke(yai_object_id_t session_id, uint64_t revoked_at, uint64_t flags);
int yai_kernel_session_close(yai_object_id_t session_id);
int yai_kernel_session_get(yai_object_id_t session_id, struct yai_kernel_session* out_session);

#endif /* YAI_KERNEL_SESSION_H */
