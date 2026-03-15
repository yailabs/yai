#include "yai/abi/errors.h"
#include "yai/kernel/capabilities.h"

int yai_kernel_capability_check(yai_kernel_capability_mask active_mask, enum yai_kernel_capability_class required_class) {
    if ((active_mask & (uint64_t)required_class) != 0) {
        return YAI_OK;
    }

    return YAI_ERR_DENIED;
}
