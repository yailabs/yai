#pragma once

#ifndef INCLUDE_NS_TYPES_H
#define INCLUDE_NS_TYPES_H

#include <stdint.h>

typedef uint64_t yai_ns_id_t;

enum yai_ns_kind {
    YAI_NS_UNKNOWN = 0,
    YAI_NS_MOUNT,
    YAI_NS_PROCESS,
    YAI_NS_NETWORK,
    YAI_NS_WORKSPACE,
    YAI_NS_RUNTIME
};

#endif
