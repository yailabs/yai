#pragma once

#ifndef INCLUDE_INPUT_TYPES_H
#define INCLUDE_INPUT_TYPES_H

#include <stdint.h>

typedef uint64_t yai_input_device_id_t;

enum yai_input_kind {
    YAI_INPUT_UNKNOWN = 0,
    YAI_INPUT_KEYBOARD,
    YAI_INPUT_POINTER,
    YAI_INPUT_TOUCH,
    YAI_INPUT_SENSOR
};

#endif
