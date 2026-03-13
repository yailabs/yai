#ifndef YAI_CONTAINER_ROOTFS_H
#define YAI_CONTAINER_ROOTFS_H

#include <stdint.h>
#include "yai/kernel/handles.h"

int yai_kernel_rootfs_project(uint64_t projection_flags, yai_rootfs_handle_t* out_rootfs_handle);

#endif /* YAI_CONTAINER_ROOTFS_H */
