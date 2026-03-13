#pragma once

#ifndef YAI_KERNEL_CAPABILITIES_H
#define YAI_KERNEL_CAPABILITIES_H

#include <stdint.h>

typedef uint64_t yai_kernel_capability_mask;

enum yai_kernel_capability_class {
    YAI_KCAP_CREATE_CONTAINER = ((uint64_t)1u << 0),
    YAI_KCAP_BIND_SESSION = ((uint64_t)1u << 1),
    YAI_KCAP_ENTER_CONTAINER = ((uint64_t)1u << 2),
    YAI_KCAP_ATTACH_MOUNT = ((uint64_t)1u << 3),
    YAI_KCAP_REQUEST_ESCAPE = ((uint64_t)1u << 4),
    YAI_KCAP_OPEN_PRIV_SHELL = ((uint64_t)1u << 5),
    YAI_KCAP_SPAWN_DAEMON_BINDING = ((uint64_t)1u << 6),
    YAI_KCAP_CONSUME_ISOLATION_CLASS = ((uint64_t)1u << 7),
    YAI_KCAP_ACCESS_IPC_CLASS = ((uint64_t)1u << 8),
    YAI_KCAP_ACCESS_RESOURCE_CLASS = ((uint64_t)1u << 9),
    YAI_KCAP_TRIGGER_CONTROL_OP = ((uint64_t)1u << 10)
};

#define YAI_CAP_PROCESS   ((uint64_t)1u << 0)
#define YAI_CAP_MEMORY    ((uint64_t)1u << 1)
#define YAI_CAP_IPC       ((uint64_t)1u << 2)
#define YAI_CAP_NET       ((uint64_t)1u << 3)
#define YAI_CAP_NAMESPACE ((uint64_t)1u << 4)

#define YAI_SUBSYS_PROC       ((uint64_t)1u << 0)
#define YAI_SUBSYS_SCHED      ((uint64_t)1u << 1)
#define YAI_SUBSYS_MM         ((uint64_t)1u << 2)
#define YAI_SUBSYS_FS         ((uint64_t)1u << 3)
#define YAI_SUBSYS_IPC        ((uint64_t)1u << 4)
#define YAI_SUBSYS_NET        ((uint64_t)1u << 5)
#define YAI_SUBSYS_SECURITY   ((uint64_t)1u << 6)
#define YAI_SUBSYS_CONTAINER  ((uint64_t)1u << 7)
#define YAI_SUBSYS_AUDIT      ((uint64_t)1u << 8)
#define YAI_SUBSYS_TRACE      ((uint64_t)1u << 9)

uint64_t yai_kernel_root_caps(void);
int yai_kernel_capability_check(yai_kernel_capability_mask active_mask, enum yai_kernel_capability_class required_class);

#endif /* YAI_KERNEL_CAPABILITIES_H */
