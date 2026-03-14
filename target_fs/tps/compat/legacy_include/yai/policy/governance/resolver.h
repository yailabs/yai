#pragma once

#include <stddef.h>

#include <yai/pol/governance/decision_map.h>
#include <yai/pol/governance/evidence_map.h>

typedef struct yai_governance_resolution_output {
  yai_governance_decision_t decision;
  yai_governance_evidence_envelope_t evidence;
  char trace_json[2048];
} yai_governance_resolution_output_t;

int yai_governance_resolve_control_call(const char *ws_id,
                                 const char *payload,
                                 const char *trace_id,
                                 yai_governance_resolution_output_t *out,
                                 char *err,
                                 size_t err_cap);
