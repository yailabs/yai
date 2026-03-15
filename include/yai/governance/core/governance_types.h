#pragma once

#ifndef INCLUDE_GOVERNANCE_TYPES_H
#define INCLUDE_GOVERNANCE_TYPES_H

#include <stdint.h>

typedef uint64_t yai_governance_record_id_t;
typedef uint64_t yai_bundle_id_t;
typedef uint64_t yai_evidence_id_t;
typedef uint64_t yai_decision_id_t;

enum yai_governance_record_kind {
    YAI_GOV_RECORD_UNKNOWN = 0,
    YAI_GOV_RECORD_BUNDLE,
    YAI_GOV_RECORD_EVIDENCE,
    YAI_GOV_RECORD_DECISION,
    YAI_GOV_RECORD_PUBLICATION,
    YAI_GOV_RECORD_REVIEW
};

#endif
