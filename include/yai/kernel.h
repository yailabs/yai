#pragma once

#ifndef YAI_KERNEL_KERNEL_H
#define YAI_KERNEL_KERNEL_H

#include <stdint.h>

/* AL-1 boundary lock: kernel is privileged minimal core only. */
#define YAI_KERNEL_BOUNDARY_LOCK_VERSION 1u

#define YAI_KERNEL_SCOPE_ABI                ((uint64_t)1u << 0)
#define YAI_KERNEL_SCOPE_STATE_LIFECYCLE    ((uint64_t)1u << 1)
#define YAI_KERNEL_SCOPE_SESSION_ADMISSION  ((uint64_t)1u << 2)
#define YAI_KERNEL_SCOPE_REGISTRY_ROOTS     ((uint64_t)1u << 3)
#define YAI_KERNEL_SCOPE_GRANTS_ROOT        ((uint64_t)1u << 4)
#define YAI_KERNEL_SCOPE_CONTAINMENT_ROOT   ((uint64_t)1u << 5)
#define YAI_KERNEL_SCOPE_CONTAINER_PRIMITIVE ((uint64_t)1u << 6)
#define YAI_KERNEL_SCOPE_POLICY_HOOKS       ((uint64_t)1u << 7)
#define YAI_KERNEL_SCOPE_TRACE_AUDIT_HOOKS  ((uint64_t)1u << 8)

#define YAI_KERNEL_ALLOWED_SCOPE_MASK \
    (YAI_KERNEL_SCOPE_ABI | \
     YAI_KERNEL_SCOPE_STATE_LIFECYCLE | \
     YAI_KERNEL_SCOPE_SESSION_ADMISSION | \
     YAI_KERNEL_SCOPE_REGISTRY_ROOTS | \
     YAI_KERNEL_SCOPE_GRANTS_ROOT | \
     YAI_KERNEL_SCOPE_CONTAINMENT_ROOT | \
     YAI_KERNEL_SCOPE_CONTAINER_PRIMITIVE | \
     YAI_KERNEL_SCOPE_POLICY_HOOKS | \
     YAI_KERNEL_SCOPE_TRACE_AUDIT_HOOKS)

static inline int yai_kernel_scope_is_allowed(uint64_t scope_mask)
{
    return (scope_mask & ~YAI_KERNEL_ALLOWED_SCOPE_MASK) == 0u;
}

void yai_kernel_bootstrap(uint64_t boot_id);
void yai_kernel_init_early(void);
void yai_kernel_tick(void);

#endif /* YAI_KERNEL_KERNEL_H */
