#pragma once

#include <stddef.h>
#include <stdint.h>

#include <yai/daemon/runtime/runtime_config.h>
#include <yai/daemon/sources/source_runtime.h>
#include <yai/daemon/runtime/runtime_paths.h>

#define YAI_EDGE_EDGE_PHASE_BOOTSTRAP "bootstrap"
#define YAI_EDGE_EDGE_PHASE_CONFIG_LOAD "config_load"
#define YAI_EDGE_EDGE_PHASE_IDENTITY_INIT "identity_init"
#define YAI_EDGE_EDGE_PHASE_SCOPE_INIT "delegated_scope_init"
#define YAI_EDGE_EDGE_PHASE_RUNTIME_START "runtime_start"
#define YAI_EDGE_EDGE_PHASE_OBSERVATION_LOOP "observation_loop"
#define YAI_EDGE_EDGE_PHASE_DEGRADED "degraded"
#define YAI_EDGE_EDGE_PHASE_DISCONNECTED "disconnected"
#define YAI_EDGE_EDGE_PHASE_SHUTDOWN "shutdown"
#define YAI_EDGE_EDGE_PHASE_STOPPED "stopped"

typedef struct yai_edge_edge_state {
  char phase[48];
  char runtime_status[32];

  char source_label[128];
  char node_identity_state[32];
  char daemon_instance_id[96];

  char owner_ref[512];
  char owner_association_state[32];
  char owner_scope_state[48];
  char delegated_policy_state[48];
  char grants_state[48];
  char source_policy_snapshot_id[128];
  char source_capability_envelope_id[128];
  char policy_snapshot_version[64];
  char distribution_target_ref[256];
  char delegated_observation_scope[128];
  char delegated_mediation_scope[128];
  char delegated_enforcement_scope[128];
  int64_t grant_issued_at_epoch;
  int64_t grant_refresh_after_epoch;
  int64_t grant_expires_at_epoch;
  int64_t snapshot_issued_at_epoch;
  int64_t snapshot_refresh_after_epoch;
  int64_t snapshot_expires_at_epoch;
  int64_t capability_issued_at_epoch;
  int64_t capability_refresh_after_epoch;
  int64_t capability_expires_at_epoch;
  int grant_revoked;
  int snapshot_revoked;
  int capability_revoked;

  char observation_state[48];
  char mediation_state[48];
  char spool_retry_state[48];
  char health_state[48];
  char connectivity_state[32];
  char freshness_state[32];
  char spool_pressure_state[32];
  char retry_pressure_state[32];
  char policy_staleness_state[48];
  char grant_validity_state[48];
  char delegated_validity_state[48];
  char delegated_refresh_state[48];
  char delegated_revoke_state[32];
  char delegated_fallback_mode[48];
  char delegated_stale_reason[96];
  char degradation_state[64];

  int owner_connected;
  uint32_t tick_count;
  int64_t started_at_epoch;
  int64_t last_tick_epoch;
  int64_t last_owner_contact_epoch;
  uint32_t spool_queued;
  uint32_t spool_retry_due;
  uint32_t spool_failed;
  uint32_t retry_consecutive_failures;
  int64_t last_observation_epoch;
  int64_t last_successful_emit_epoch;
} yai_edge_edge_state_t;

int yai_edge_edge_state_init(yai_edge_edge_state_t *state,
                               const yai_edge_config_t *cfg,
                               const yai_edge_paths_t *paths,
                               const char *instance_id);
int yai_edge_edge_state_set_phase(yai_edge_edge_state_t *state,
                                    const char *phase);
int yai_edge_edge_state_set_runtime_status(yai_edge_edge_state_t *state,
                                             const char *status);
int yai_edge_edge_state_refresh_from_local(yai_edge_edge_state_t *state,
                                             const yai_edge_local_runtime_t *local,
                                             uint32_t tick_count);
int yai_edge_edge_state_json(const yai_edge_edge_state_t *state,
                               char *out,
                               size_t out_cap);
