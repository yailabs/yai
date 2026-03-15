#pragma once

#ifndef INCLUDE_REGISTRY_TYPES_H
#define INCLUDE_REGISTRY_TYPES_H

#include <stdint.h>

typedef uint64_t yai_registry_record_id_t;

enum yai_registry_class_kind {
    YAI_REGISTRY_CLASS_UNKNOWN = 0,
    YAI_REGISTRY_CLASS_DEVICE,
    YAI_REGISTRY_CLASS_TASK,
    YAI_REGISTRY_CLASS_POLICY,
    YAI_REGISTRY_CLASS_WORKSPACE
};

#endif
