#pragma once

#ifndef INCLUDE_POLICY_DECISION_H
#define INCLUDE_POLICY_DECISION_H

#include <yai/policy/types.h>
#include <yai/policy/result.h>

struct yai_policy_decision {
    yai_policy_eval_id_t eval_id;
    yai_policy_id_t policy_id;
    enum yai_policy_result result;
};

#endif
