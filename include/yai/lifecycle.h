#pragma once

#ifndef YAI_KERNEL_LIFECYCLE_H
#define YAI_KERNEL_LIFECYCLE_H

#include <stdint.h>

enum yai_kernel_lifecycle_state_id {
    YAI_KERNEL_STATE_BOOTING = 0,
    YAI_KERNEL_STATE_INITIALIZING = 1,
    YAI_KERNEL_STATE_READY = 2,
    YAI_KERNEL_STATE_DEGRADED = 3,
    YAI_KERNEL_STATE_RECOVERY = 4,
    YAI_KERNEL_STATE_SHUTTING_DOWN = 5,
    YAI_KERNEL_STATE_HALTED = 6
};

enum yai_kernel_transition_result {
    YAI_KERNEL_TRANSITION_OK = 0,
    YAI_KERNEL_TRANSITION_INVALID = 1,
    YAI_KERNEL_TRANSITION_BLOCKED = 2,
    YAI_KERNEL_TRANSITION_NOT_READY = 3
};

#define YAI_KERNEL_READY_PROC                ((uint64_t)1u << 0)
#define YAI_KERNEL_READY_SCHED               ((uint64_t)1u << 1)
#define YAI_KERNEL_READY_MM                  ((uint64_t)1u << 2)
#define YAI_KERNEL_READY_FS                  ((uint64_t)1u << 3)
#define YAI_KERNEL_READY_IPC                 ((uint64_t)1u << 4)
#define YAI_KERNEL_READY_SECURITY            ((uint64_t)1u << 5)
#define YAI_KERNEL_READY_CONTAINER_PRIMITIVE ((uint64_t)1u << 6)
#define YAI_KERNEL_READY_AUDIT               ((uint64_t)1u << 7)
#define YAI_KERNEL_READY_REGISTRY            ((uint64_t)1u << 8)
#define YAI_KERNEL_READY_SESSION_ADMISSION   ((uint64_t)1u << 9)
#define YAI_KERNEL_READY_POLICY_GRANTS       ((uint64_t)1u << 10)

#define YAI_KERNEL_READY_MINIMAL_MASK \
    (YAI_KERNEL_READY_PROC | \
     YAI_KERNEL_READY_SCHED | \
     YAI_KERNEL_READY_MM | \
     YAI_KERNEL_READY_FS | \
     YAI_KERNEL_READY_IPC | \
     YAI_KERNEL_READY_SECURITY | \
     YAI_KERNEL_READY_CONTAINER_PRIMITIVE | \
     YAI_KERNEL_READY_AUDIT | \
     YAI_KERNEL_READY_REGISTRY | \
     YAI_KERNEL_READY_SESSION_ADMISSION | \
     YAI_KERNEL_READY_POLICY_GRANTS)

int yai_kernel_boot_begin(uint64_t boot_id, uint64_t entered_at);
int yai_kernel_mark_subsystem_ready(uint64_t readiness_flag, int ready);

int yai_kernel_enter_ready(uint32_t reason, uint64_t entered_at);
int yai_kernel_enter_degraded(uint32_t reason, uint64_t entered_at);
int yai_kernel_enter_recovery(uint32_t reason, uint64_t entered_at);
int yai_kernel_begin_shutdown(uint32_t reason, uint64_t entered_at);
int yai_kernel_halt(uint32_t reason, uint64_t entered_at);

int yai_kernel_can_admit_sessions(void);
int yai_kernel_can_create_container(void);
int yai_kernel_can_bind_container(void);
int yai_kernel_can_issue_grants(void);

int yai_kernel_lifecycle_transition_allowed(
    enum yai_kernel_lifecycle_state_id from,
    enum yai_kernel_lifecycle_state_id to,
    enum yai_kernel_transition_result* out_result);

#endif /* YAI_KERNEL_LIFECYCLE_H */
