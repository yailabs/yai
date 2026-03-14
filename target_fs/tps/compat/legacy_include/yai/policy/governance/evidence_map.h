#pragma once

#include <stddef.h>

#include <yai/pol/governance/decision_map.h>

typedef struct yai_governance_evidence_envelope {
  char trace_id[64];
  char decision_id[64];
  char domain_id[64];
  char family_id[64];
  char specialization_id[96];
  char final_effect[32];
  char provider[64];
  char resource[64];
  char authority_context[64];
  int review_trace_required;
  int retention_required;
  int provenance_required;
  int approval_chain_required;
  int dependency_chain_required;
  int lawful_basis_required;
  int oversight_trace_required;
} yai_governance_evidence_envelope_t;

int yai_governance_decision_to_evidence(const yai_governance_decision_t *decision,
                                 const char *trace_id,
                                 const char *provider,
                                 const char *resource,
                                 const char *authority_context,
                                 yai_governance_evidence_envelope_t *out);

int yai_governance_evidence_to_record_blob(const yai_governance_decision_t *decision,
                                    const yai_governance_evidence_envelope_t *evidence,
                                    char *out,
                                    size_t out_cap);
