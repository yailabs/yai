#pragma once

#ifndef INCLUDE_MOUNT_FLAGS_H
#define INCLUDE_MOUNT_FLAGS_H

enum yai_mount_flags {
    YAI_MOUNT_READONLY   = 1u << 0,
    YAI_MOUNT_NOEXEC     = 1u << 1,
    YAI_MOUNT_NOSUID     = 1u << 2,
    YAI_MOUNT_NODEV      = 1u << 3,
    YAI_MOUNT_EPHEMERAL  = 1u << 4
};

#endif
