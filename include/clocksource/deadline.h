#pragma once

#ifndef INCLUDE_CLOCKSOURCE_DEADLINE_H
#define INCLUDE_CLOCKSOURCE_DEADLINE_H

#include <clocksource/types.h>

struct yai_clock_deadline {
    yai_clock_tick_t expires_at;
};

#endif
