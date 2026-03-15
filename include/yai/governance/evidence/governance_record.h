#pragma once

#ifndef INCLUDE_GOVERNANCE_RECORD_H
#define INCLUDE_GOVERNANCE_RECORD_H

#include <yai/governance/types.h>
#include <yai/governance/status.h>

struct yai_governance_record {
    yai_governance_record_id_t id;
    enum yai_governance_record_kind kind;
    enum yai_governance_status status;
    const char *name;
};

#endif
