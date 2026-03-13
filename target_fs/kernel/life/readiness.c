#include "yai/abi/errors.h"
#include "yai/kernel/lifecycle.h"
#include "yai/kernel/state.h"

int yai_kernel_mark_subsystem_ready(uint64_t readiness_flag, int ready) {
    return yai_kernel_state_set_readiness(readiness_flag, ready);
}

int yai_kernel_can_admit_sessions(void) {
    const struct yai_kernel_state* state = yai_kernel_state_get();

    if (state->lifecycle.current_state != YAI_KERNEL_STATE_READY) {
        return YAI_ERR_DENIED;
    }

    return yai_kernel_state_has_readiness(
        YAI_KERNEL_READY_SESSION_ADMISSION |
        YAI_KERNEL_READY_POLICY_GRANTS);
}

int yai_kernel_can_create_container(void) {
    const struct yai_kernel_state* state = yai_kernel_state_get();

    if (state->lifecycle.current_state != YAI_KERNEL_STATE_READY &&
        state->lifecycle.current_state != YAI_KERNEL_STATE_DEGRADED) {
        return YAI_ERR_DENIED;
    }

    return yai_kernel_state_has_readiness(
        YAI_KERNEL_READY_CONTAINER_PRIMITIVE |
        YAI_KERNEL_READY_SECURITY |
        YAI_KERNEL_READY_FS);
}

int yai_kernel_can_bind_container(void) {
    const struct yai_kernel_state* state = yai_kernel_state_get();

    if (state->lifecycle.current_state != YAI_KERNEL_STATE_READY) {
        return YAI_ERR_DENIED;
    }

    return yai_kernel_state_has_readiness(
        YAI_KERNEL_READY_CONTAINER_PRIMITIVE |
        YAI_KERNEL_READY_SESSION_ADMISSION);
}

int yai_kernel_can_issue_grants(void) {
    const struct yai_kernel_state* state = yai_kernel_state_get();

    if (state->lifecycle.current_state != YAI_KERNEL_STATE_READY &&
        state->lifecycle.current_state != YAI_KERNEL_STATE_DEGRADED) {
        return YAI_ERR_DENIED;
    }

    return yai_kernel_state_has_readiness(
        YAI_KERNEL_READY_REGISTRY |
        YAI_KERNEL_READY_POLICY_GRANTS);
}
