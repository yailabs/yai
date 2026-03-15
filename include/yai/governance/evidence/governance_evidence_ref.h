#pragma once

#ifndef INCLUDE_GOVERNANCE_EVIDENCE_REF_H
#define INCLUDE_GOVERNANCE_EVIDENCE_REF_H

#include <yai/governance/types.h>

struct yai_evidence_ref {
    yai_evidence_id_t evidence_id;
    const char *locator;
};

#endif
