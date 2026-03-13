#ifndef YAI_CONTAINER_CGROUP_H
#define YAI_CONTAINER_CGROUP_H

#include <stdint.h>
#include "yai/kernel/handles.h"

int yai_kernel_cgroup_attach(yai_cgroup_id_t cgroup_id, uint64_t limits_flags);

#endif /* YAI_CONTAINER_CGROUP_H */
