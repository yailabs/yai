#include "yai/abi/errors.h"
#include "yai/kernel/lifecycle.h"

int yai_kernel_lifecycle_transition_allowed(
    enum yai_kernel_lifecycle_state_id from,
    enum yai_kernel_lifecycle_state_id to,
    enum yai_kernel_transition_result* out_result) {
    int allowed = 0;

    if (out_result == 0) {
        return YAI_ERR_INVALID;
    }

    switch (from) {
        case YAI_KERNEL_STATE_BOOTING:
            allowed = (to == YAI_KERNEL_STATE_INITIALIZING);
            break;
        case YAI_KERNEL_STATE_INITIALIZING:
            allowed = (to == YAI_KERNEL_STATE_READY || to == YAI_KERNEL_STATE_DEGRADED);
            break;
        case YAI_KERNEL_STATE_READY:
            allowed = (to == YAI_KERNEL_STATE_DEGRADED || to == YAI_KERNEL_STATE_SHUTTING_DOWN);
            break;
        case YAI_KERNEL_STATE_DEGRADED:
            allowed = (to == YAI_KERNEL_STATE_RECOVERY || to == YAI_KERNEL_STATE_SHUTTING_DOWN);
            break;
        case YAI_KERNEL_STATE_RECOVERY:
            allowed = (to == YAI_KERNEL_STATE_READY || to == YAI_KERNEL_STATE_SHUTTING_DOWN);
            break;
        case YAI_KERNEL_STATE_SHUTTING_DOWN:
            allowed = (to == YAI_KERNEL_STATE_HALTED);
            break;
        case YAI_KERNEL_STATE_HALTED:
            allowed = 0;
            break;
        default:
            *out_result = YAI_KERNEL_TRANSITION_INVALID;
            return YAI_ERR_INVALID;
    }

    *out_result = allowed ? YAI_KERNEL_TRANSITION_OK : YAI_KERNEL_TRANSITION_INVALID;
    return YAI_OK;
}
