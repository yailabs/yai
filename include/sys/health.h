#pragma once

#ifndef INCLUDE_SYS_HEALTH_H
#define INCLUDE_SYS_HEALTH_H

struct yai_system_health {
    int healthy;
    unsigned int degraded_subsystems;
};

#endif
