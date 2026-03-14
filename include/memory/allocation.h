#pragma once

#ifndef INCLUDE_MEMORY_ALLOCATION_H
#define INCLUDE_MEMORY_ALLOCATION_H

#include <memory/flags.h>

struct yai_memory_allocation_request {
    unsigned long size;
    unsigned int flags;
};

#endif
