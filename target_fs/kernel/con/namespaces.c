#include "yai/abi/errors.h"
#include <yai/container/namespace.h>

static uint64_t g_next_ns = 1;

int yai_kernel_namespace_allocate_set(yai_namespace_handles_t* out_handles) {
    if (out_handles == 0) {
        return YAI_ERR_INVALID;
    }

    out_handles->mount_ns = g_next_ns++;
    out_handles->pid_ns = g_next_ns++;
    out_handles->net_ns = g_next_ns++;
    out_handles->ipc_ns = g_next_ns++;

    return YAI_OK;
}
