#pragma once

#ifndef YAI_KERNEL_HANDLES_H
#define YAI_KERNEL_HANDLES_H

#include <stdint.h>

enum yai_registry_handle {
    YAI_REGISTRY_HANDLE_INVALID = 0,
    YAI_REGISTRY_HANDLE_KERNEL = 1,
    YAI_REGISTRY_HANDLE_CONTAINER = 2,
    YAI_REGISTRY_HANDLE_SESSION = 3,
    YAI_REGISTRY_HANDLE_CAPABILITY = 4,
    YAI_REGISTRY_HANDLE_CONTAINMENT = 5,
    YAI_REGISTRY_HANDLE_TRACE = 6,
    YAI_REGISTRY_HANDLE_GRANTS = 7
};

typedef uint64_t yai_container_handle_t;
typedef uint64_t yai_session_handle_t;
typedef uint64_t yai_rootfs_handle_t;

typedef uint64_t yai_mount_ns_id_t;
typedef uint64_t yai_pid_ns_id_t;
typedef uint64_t yai_net_ns_id_t;
typedef uint64_t yai_ipc_ns_id_t;
typedef uint64_t yai_cgroup_id_t;

typedef struct yai_namespace_handles {
    yai_mount_ns_id_t mount_ns;
    yai_pid_ns_id_t pid_ns;
    yai_net_ns_id_t net_ns;
    yai_ipc_ns_id_t ipc_ns;
} yai_namespace_handles_t;

#endif /* YAI_KERNEL_HANDLES_H */
