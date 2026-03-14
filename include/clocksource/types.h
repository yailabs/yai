#pragma once

#ifndef INCLUDE_CLOCKSOURCE_TYPES_H
#define INCLUDE_CLOCKSOURCE_TYPES_H

#include <stdint.h>

typedef uint64_t yai_clocksource_id_t;
typedef uint64_t yai_clock_tick_t;

enum yai_clocksource_kind {
    YAI_CLOCKSOURCE_UNKNOWN = 0,
    YAI_CLOCKSOURCE_MONOTONIC,
    YAI_CLOCKSOURCE_REALTIME,
    YAI_CLOCKSOURCE_DEADLINE,
    YAI_CLOCKSOURCE_WATCHDOG
};

#endif
