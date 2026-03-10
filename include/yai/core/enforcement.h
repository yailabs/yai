#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>

#include <yai/core/authority.h>
#include <yai/core/lifecycle.h>
#include <yai/law/resolver.h>
#include <yai/protocol/rpc_runtime.h>

typedef struct yai_hardened_config {
  char storage_backend[32];
  uint16_t max_parallel_agents;
  bool enforce_tla_safety;
} yai_hardened_config_t;

/* Transitional rpc.v1 envelope validation errors. */
#define YAI_E_OK                0
#define YAI_E_BAD_ARG          -1
#define YAI_E_BAD_VERSION      -2
#define YAI_E_MISSING_WS       -3
#define YAI_E_WS_MISMATCH      -4
#define YAI_E_MISSING_TYPE     -5
#define YAI_E_TYPE_NOT_ALLOWED -6
#define YAI_E_PRIV_REQUIRED    -7
#define YAI_E_ROLE_REQUIRED    -8
#define YAI_E_HANDSHAKE_REQUIRED -9

bool yai_config_enforce_limits(yai_hardened_config_t *cfg);
int yai_validate_envelope_v1(const char *line, const char *expected_ws, char *out_request_type, size_t req_cap);

typedef struct yai_enforcement_decision {
  char status[16];
  char code[48];
  char reason[128];
  char review_state[32];
  char authority_constraints[256];
  int authority_constraint_count;
  yai_authority_decision_t authority_decision;
  int runtime_bound;
} yai_enforcement_decision_t;

int yai_enforcement_finalize_control_call(const yai_rpc_envelope_t *env,
                                          const char *workspace_id,
                                          const yai_law_resolution_output_t *law_out,
                                          const yai_runtime_capability_state_t *caps,
                                          yai_enforcement_decision_t *out,
                                          char *err,
                                          size_t err_cap);
