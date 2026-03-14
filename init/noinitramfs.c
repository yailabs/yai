// SPDX-License-Identifier: Apache-2.0

#include "noinitramfs.h"

#include <yai/log.h>
#include <container/container_rootfs.h>

int yai_init_default_rootfs(void)
{
    int rc;

    rc = yai_rootfs_mkdir("/dev", 0755);
    if (rc < 0) {
        yai_log_error("rootfs: failed to create /dev");
        return rc;
    }

    rc = yai_rootfs_mknod_console("/dev/console");
    if (rc < 0) {
        yai_log_error("rootfs: failed to create /dev/console");
        return rc;
    }

    rc = yai_rootfs_mkdir("/root", 0700);
    if (rc < 0) {
        yai_log_error("rootfs: failed to create /root");
        return rc;
    }

    rc = yai_rootfs_mkdir("/run", 0755);
    if (rc < 0) {
        yai_log_error("rootfs: failed to create /run");
        return rc;
    }

    return 0;
}