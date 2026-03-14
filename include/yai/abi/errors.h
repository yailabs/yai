#ifndef YAI_KERNEL_ABI_ERRORS_H
#define YAI_KERNEL_ABI_ERRORS_H

enum yai_status {
    YAI_OK = 0,
    YAI_ERR_INVALID = -1,
    YAI_ERR_DENIED = -2,
    YAI_ERR_NOT_FOUND = -3,
    YAI_ERR_UNSUPPORTED = -4,
    YAI_ERR_BUSY = -5
};

#endif /* YAI_KERNEL_ABI_ERRORS_H */
