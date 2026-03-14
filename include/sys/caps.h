#pragma once

#ifndef INCLUDE_SYS_CAPS_H
#define INCLUDE_SYS_CAPS_H

enum yai_system_capability {
    YAI_SYS_CAP_NONE        = 0,
    YAI_SYS_CAP_STORAGE     = 1u << 0,
    YAI_SYS_CAP_NETWORK     = 1u << 1,
    YAI_SYS_CAP_POLICY      = 1u << 2,
    YAI_SYS_CAP_SUPERVISION = 1u << 3,
    YAI_SYS_CAP_TRACING     = 1u << 4
};

#endif
