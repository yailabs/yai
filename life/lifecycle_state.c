#include "yai/abi/errors.h"
#include "yai/kernel/lifecycle.h"
#include "yai/kernel/registry.h"
#include "yai/kernel/state.h"

static int yai_kernel_transition_to(
    enum yai_kernel_lifecycle_state_id target,
    uint32_t reason,
    uint64_t entered_at) {
    const struct yai_kernel_state* state = yai_kernel_state_get();
    enum yai_kernel_transition_result result;
    int rc;

    rc = yai_kernel_lifecycle_transition_allowed(state->lifecycle.current_state, target, &result);
    if (rc != YAI_OK) {
        return rc;
    }

    if (result != YAI_KERNEL_TRANSITION_OK) {
        return YAI_ERR_DENIED;
    }

    return yai_kernel_state_set_lifecycle(target, reason, entered_at);
}

int yai_kernel_boot_begin(uint64_t boot_id, uint64_t entered_at) {
    struct yai_kernel_registry_roots roots;
    int rc;

    yai_kernel_registry_bootstrap(&roots);
    yai_kernel_state_init(boot_id, &roots);

    rc = yai_kernel_state_set_lifecycle(YAI_KERNEL_STATE_BOOTING, 0, entered_at);
    if (rc != YAI_OK) {
        return rc;
    }

    rc = yai_kernel_transition_to(YAI_KERNEL_STATE_INITIALIZING, 0, entered_at);
    if (rc != YAI_OK) {
        return rc;
    }

    yai_kernel_mark_subsystem_ready(YAI_KERNEL_READY_REGISTRY, 1);
    return YAI_OK;
}

int yai_kernel_enter_ready(uint32_t reason, uint64_t entered_at) {
    int rc = yai_kernel_state_has_readiness(YAI_KERNEL_READY_MINIMAL_MASK);
    if (rc != YAI_OK) {
        return YAI_ERR_DENIED;
    }

    return yai_kernel_transition_to(YAI_KERNEL_STATE_READY, reason, entered_at);
}

int yai_kernel_enter_degraded(uint32_t reason, uint64_t entered_at) {
    return yai_kernel_transition_to(YAI_KERNEL_STATE_DEGRADED, reason, entered_at);
}

int yai_kernel_enter_recovery(uint32_t reason, uint64_t entered_at) {
    return yai_kernel_transition_to(YAI_KERNEL_STATE_RECOVERY, reason, entered_at);
}

int yai_kernel_begin_shutdown(uint32_t reason, uint64_t entered_at) {
    return yai_kernel_transition_to(YAI_KERNEL_STATE_SHUTTING_DOWN, reason, entered_at);
}

int yai_kernel_halt(uint32_t reason, uint64_t entered_at) {
    return yai_kernel_transition_to(YAI_KERNEL_STATE_HALTED, reason, entered_at);
}
