#pragma once

#ifndef INCLUDE_INPUT_SOURCE_H
#define INCLUDE_INPUT_SOURCE_H

#include <input/types.h>

struct yai_input_source {
    yai_input_device_id_t device_id;
    enum yai_input_kind kind;
};

#endif
