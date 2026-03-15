#pragma once

#ifndef INCLUDE_POLICY_TYPES_H
#define INCLUDE_POLICY_TYPES_H

#include <stdint.h>

typedef uint64_t yai_policy_id_t;
typedef uint64_t yai_policy_target_id_t;
typedef uint64_t yai_policy_eval_id_t;

enum yai_policy_kind {
    YAI_POLICY_UNKNOWN = 0,
    YAI_POLICY_ACCESS,
    YAI_POLICY_CONTAINMENT,
    YAI_POLICY_GRANT,
    YAI_POLICY_OVERLAY,
    YAI_POLICY_RUNTIME
};

#endif
