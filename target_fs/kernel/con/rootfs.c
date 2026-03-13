#include "yai/abi/errors.h"
#include <yai/container/rootfs.h>

static yai_rootfs_handle_t g_next_rootfs = 1;

int yai_kernel_rootfs_project(uint64_t projection_flags, yai_rootfs_handle_t* out_rootfs_handle) {
    (void)projection_flags;

    if (out_rootfs_handle == 0) {
        return YAI_ERR_INVALID;
    }

    *out_rootfs_handle = g_next_rootfs++;
    return YAI_OK;
}
