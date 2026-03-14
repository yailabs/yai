#pragma once

#ifndef INCLUDE_CLOCKSOURCE_EVENT_H
#define INCLUDE_CLOCKSOURCE_EVENT_H

#include <clocksource/types.h>

struct yai_clock_event {
    yai_clocksource_id_t source_id;
    yai_clock_tick_t tick;
};

#endif
