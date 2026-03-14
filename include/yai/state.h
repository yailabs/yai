#pragma once

#ifndef YAI_KERNEL_STATE_H
#define YAI_KERNEL_STATE_H

#include <stdint.h>

#include <yai/container/lifecycle.h>
#include "lifecycle.h"
#include "registry.h"

struct yai_kernel_lifecycle_state {
    enum yai_kernel_lifecycle_state_id current_state;
    enum yai_kernel_lifecycle_state_id previous_state;
    uint64_t readiness_flags;
    uint32_t last_transition_reason;
    uint64_t entered_at;
    uint64_t flags;
};

struct yai_kernel_capability_state {
    uint64_t root_caps;
    uint64_t ready_subsystems;
};

struct yai_kernel_state {
    uint64_t boot_id;
    uint64_t tick;
    uint64_t mode_flags;

    struct yai_kernel_lifecycle_state lifecycle;
    struct yai_kernel_capability_state capability;
    struct yai_kernel_registry_roots roots;

    uint64_t admitted_sessions;
    uint64_t active_containers;
    uint64_t audit_channels;
};

void yai_kernel_state_init(uint64_t boot_id, const struct yai_kernel_registry_roots* roots);

int yai_kernel_state_set_lifecycle(
    enum yai_kernel_lifecycle_state_id next_state,
    uint32_t reason,
    uint64_t entered_at);

int yai_kernel_state_set_readiness(uint64_t readiness_flag, int ready);
int yai_kernel_state_has_readiness(uint64_t readiness_mask);

int yai_kernel_state_set_subsystem_ready(uint64_t subsystem_flag, int ready);

void yai_kernel_state_inc_sessions(void);
void yai_kernel_state_inc_containers(void);
void yai_kernel_state_inc_audit_channels(void);

const struct yai_kernel_state* yai_kernel_state_get(void);

#endif /* YAI_KERNEL_STATE_H */
