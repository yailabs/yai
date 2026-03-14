#pragma once

#ifndef INCLUDE_MEMORY_FLAGS_H
#define INCLUDE_MEMORY_FLAGS_H

enum yai_memory_flags {
    YAI_MEMORY_ZEROED    = 1u << 0,
    YAI_MEMORY_PINNED    = 1u << 1,
    YAI_MEMORY_SHARED    = 1u << 2,
    YAI_MEMORY_EXECUTABLE = 1u << 3
};

#endif
