#ifndef YAI_KERNEL_ABI_SYSCALL_H
#define YAI_KERNEL_ABI_SYSCALL_H

#include <stdint.h>
#include "errors.h"

enum yai_syscall_id {
    YAI_SYSCALL_NOP = 0,
    YAI_SYSCALL_PROCESS_SPAWN = 1,
    YAI_SYSCALL_CHANNEL_OPEN = 2,
    YAI_SYSCALL_CAP_QUERY = 3
};

int yai_syscall_dispatch(uint32_t id, uint64_t a0, uint64_t a1, uint64_t a2, uint64_t a3);

#endif /* YAI_KERNEL_ABI_SYSCALL_H */
