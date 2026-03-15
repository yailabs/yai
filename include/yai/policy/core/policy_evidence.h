#pragma once

#ifndef INCLUDE_POLICY_EVIDENCE_H
#define INCLUDE_POLICY_EVIDENCE_H

#include <yai/governance/types.h>

struct yai_policy_evidence_binding {
    yai_evidence_id_t evidence_id;
    const char *binding_reason;
};

#endif
