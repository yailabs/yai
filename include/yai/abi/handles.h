#ifndef YAI_KERNEL_ABI_HANDLES_H
#define YAI_KERNEL_ABI_HANDLES_H

#include "types.h"

enum yai_handle_kind {
    YAI_HANDLE_INVALID = 0,
    YAI_HANDLE_PROCESS = 1,
    YAI_HANDLE_THREAD = 2,
    YAI_HANDLE_CHANNEL = 3,
    YAI_HANDLE_NAMESPACE = 4
};

#endif /* YAI_KERNEL_ABI_HANDLES_H */
