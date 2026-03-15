#include "yai/abi/errors.h"
#include <yai/con/cgroup.h>
#include <yai/con/limits.h>

int yai_kernel_limits_validate(uint64_t limits_flags) {
    (void)limits_flags;
    return YAI_OK;
}

int yai_kernel_cgroup_attach(yai_cgroup_id_t cgroup_id, uint64_t limits_flags) {
    if (cgroup_id == 0) {
        return YAI_ERR_INVALID;
    }

    return yai_kernel_limits_validate(limits_flags);
}
