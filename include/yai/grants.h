#pragma once

#ifndef YAI_KERNEL_GRANTS_H
#define YAI_KERNEL_GRANTS_H

#include <stdint.h>

#include "capabilities.h"
#include "objects.h"

enum yai_kernel_grant_state {
    YAI_KERNEL_GRANT_PENDING = 0,
    YAI_KERNEL_GRANT_ACTIVE = 1,
    YAI_KERNEL_GRANT_SUSPENDED = 2,
    YAI_KERNEL_GRANT_REVOKED = 3,
    YAI_KERNEL_GRANT_EXPIRED = 4,
    YAI_KERNEL_GRANT_DENIED = 5
};

struct yai_kernel_grant {
    yai_object_id_t grant_id;
    yai_object_id_t subject_handle;
    enum yai_kernel_capability_class capability_class;
    yai_object_id_t scope_handle;
    enum yai_kernel_grant_state validity_state;
    uint64_t issued_at;
    uint64_t expires_at;
    uint64_t revoked_at;
    uint64_t flags;
};

struct yai_kernel_grant_request {
    yai_object_id_t grant_id;
    yai_object_id_t subject_handle;
    enum yai_kernel_capability_class capability_class;
    yai_object_id_t scope_handle;
    uint64_t issued_at;
    uint64_t expires_at;
    uint64_t flags;
};

int yai_kernel_grant_issue(const struct yai_kernel_grant_request* request, struct yai_kernel_grant* out_grant);
int yai_kernel_grant_activate(yai_object_id_t grant_id);
int yai_kernel_grant_suspend(yai_object_id_t grant_id, uint64_t flags);
int yai_kernel_grant_revoke(yai_object_id_t grant_id, uint64_t revoked_at, uint64_t flags);
int yai_kernel_grant_expire(yai_object_id_t grant_id, uint64_t expired_at);

int yai_kernel_grant_check(
    yai_object_id_t subject_handle,
    enum yai_kernel_capability_class capability_class,
    yai_object_id_t scope_handle,
    struct yai_kernel_grant* out_grant);

#endif /* YAI_KERNEL_GRANTS_H */
