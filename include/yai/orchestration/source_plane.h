#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* YD-1 canonical topology lock: distributed acquisition, centralized control. */
#define YAI_SOURCE_PLANE_TOPOLOGY "distributed-acquisition-centralized-control-v1"
#define YAI_SOURCE_PLANE_OWNER_RUNTIME "yai"
#define YAI_SOURCE_PLANE_EDGE_BINARY "yai-edge"
#define YAI_SOURCE_PLANE_EDGE_ROLE "subordinate-edge-runtime"

/* Returns 1 when owner-runtime centralized control invariant is active. */
int yai_source_plane_owner_canonical(void);

/* Returns canonical topology id string. */
const char *yai_source_plane_topology_id(void);

typedef struct yai_source_plane_mediation_state {
  int owner_canonical;
  int transport_ready;
  int network_gate_ready;
  int resource_gate_ready;
  int storage_gate_ready;
  char route[48];
  char stage[64];
} yai_source_plane_mediation_state_t;

/* RF-0.3 delegated edge enforcement semantics. */
typedef enum yai_edge_enforcement_mode {
  YAI_EDGE_ENFORCE_OBSERVE_ONLY = 0,
  YAI_EDGE_ENFORCE_POST_EVENT = 1,
  YAI_EDGE_ENFORCE_PREVENTIVE = 2,
  YAI_EDGE_ENFORCE_ESCALATED = 3
} yai_edge_enforcement_mode_t;

typedef enum yai_edge_enforcement_outcome {
  YAI_EDGE_OUTCOME_OBSERVE_ONLY = 0,
  YAI_EDGE_OUTCOME_ALLOW = 1,
  YAI_EDGE_OUTCOME_BLOCK = 2,
  YAI_EDGE_OUTCOME_HOLD = 3,
  YAI_EDGE_OUTCOME_EXECUTE = 4,
  YAI_EDGE_OUTCOME_ESCALATE = 5,
  YAI_EDGE_OUTCOME_DEFER = 6,
  YAI_EDGE_OUTCOME_DENY_MISSING_SCOPE = 7,
  YAI_EDGE_OUTCOME_DENY_EXPIRED_GRANT = 8
} yai_edge_enforcement_outcome_t;

/* Returns canonical exec route id for a source-plane command. */
const char *yai_exec_source_plane_route_for_command(const char *command_id);

/*
 * Preflight mediation for source-plane operations.
 * This is the guardrail boundary that keeps source owner/edge mediation
 * in exec before handing off to owner canonical runtime persistence/truth.
 */
int yai_exec_source_plane_prepare(const char *command_id,
                                  yai_source_plane_mediation_state_t *out_state,
                                  char *err,
                                  size_t err_cap);

#ifdef __cplusplus
}
#endif
