#pragma once

#include <stddef.h>

typedef struct yai_governance_classification_ctx {
  char ws_id[64];
  char command[128];
  char action[64];
  char provider[64];
  char resource[64];
  char protocol[32];
  char workspace_mode[32];
  char declared_family[64];
  char declared_specialization[96];
  int has_params_hash;
  int black_box_mode;
  int has_authority_contract;
  int has_repro_context;
  int has_dataset_ref;
  int has_publication_intent;
  int has_locked_parameters;
  int has_result_ref;
  int has_retrieve_intent;
  int has_egress_intent;
  int has_commentary_intent;
  int has_distribution_intent;
  int has_sink_ref;
  int sink_trusted;
  int sink_external;
} yai_governance_classification_ctx_t;

int yai_governance_classify_event(const char *ws_id,
                           const char *payload,
                           yai_governance_classification_ctx_t *out);
