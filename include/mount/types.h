#pragma once

#ifndef INCLUDE_MOUNT_TYPES_H
#define INCLUDE_MOUNT_TYPES_H

#include <stdint.h>

typedef uint64_t yai_mount_graph_id_t;

enum yai_mount_kind {
    YAI_MOUNT_UNKNOWN = 0,
    YAI_MOUNT_ROOT,
    YAI_MOUNT_BIND,
    YAI_MOUNT_OVERLAY,
    YAI_MOUNT_RUNTIME
};

#endif
