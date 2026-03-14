#pragma once

#ifndef INCLUDE_MEMORY_STATS_H
#define INCLUDE_MEMORY_STATS_H

struct yai_memory_stats {
    unsigned long bytes_total;
    unsigned long bytes_used;
    unsigned long bytes_free;
};

#endif
