#pragma once

#ifndef INCLUDE_POLICY_TARGET_H
#define INCLUDE_POLICY_TARGET_H

#include <yai/policy/types.h>

struct yai_policy_target {
    yai_policy_target_id_t target_id;
    const char *target_kind;
};

#endif
