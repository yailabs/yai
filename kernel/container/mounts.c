#include "yai/abi/errors.h"
#include "yai/kernel/mount_policy.h"

int yai_kernel_mounts_validate_policy(enum yai_mount_policy_class mount_policy, uint64_t flags) {
    (void)flags;

    if (mount_policy < YAI_MOUNT_POLICY_READONLY || mount_policy > YAI_MOUNT_POLICY_PRIVILEGED_RW) {
        return YAI_ERR_INVALID;
    }

    return YAI_OK;
}
