#pragma once

#include <stddef.h>
#include <stdint.h>

#include <yai/daemon/config.h>
#include <yai/daemon/action_point.h>
#include <yai/daemon/paths.h>

#define YAI_DAEMON_BINDING_STATUS_CONFIGURED "configured"
#define YAI_DAEMON_BINDING_STATUS_ACTIVE "active"
#define YAI_DAEMON_BINDING_STATUS_DEGRADED "degraded"
#define YAI_DAEMON_BINDING_STATUS_INVALID "invalid"

#define YAI_DAEMON_UNIT_STATUS_DISCOVERED "discovered"
#define YAI_DAEMON_UNIT_STATUS_QUEUED "queued"
#define YAI_DAEMON_UNIT_STATUS_EMITTING "emitting"
#define YAI_DAEMON_UNIT_STATUS_DELIVERED "delivered"
#define YAI_DAEMON_UNIT_STATUS_RETRY_DUE "retry_due"
#define YAI_DAEMON_UNIT_STATUS_FAILED "failed_terminal"

#define YAI_DAEMON_HEALTH_STARTING "starting"
#define YAI_DAEMON_HEALTH_READY "ready"
#define YAI_DAEMON_HEALTH_DEGRADED "degraded"
#define YAI_DAEMON_HEALTH_DISCONNECTED "disconnected"
#define YAI_DAEMON_HEALTH_STOPPING "stopping"

#define YAI_DAEMON_CONNECTIVITY_CONNECTED "connected"
#define YAI_DAEMON_CONNECTIVITY_DISCONNECTED "disconnected"
#define YAI_DAEMON_CONNECTIVITY_UNCONFIGURED "unconfigured"

#define YAI_DAEMON_FRESHNESS_FRESH "fresh"
#define YAI_DAEMON_FRESHNESS_AGING "aging"
#define YAI_DAEMON_FRESHNESS_STALE "stale"
#define YAI_DAEMON_FRESHNESS_UNKNOWN "unknown"

#define YAI_DAEMON_PRESSURE_LOW "low"
#define YAI_DAEMON_PRESSURE_MEDIUM "medium"
#define YAI_DAEMON_PRESSURE_HIGH "high"

#define YAI_DAEMON_POLICY_STALENESS_PENDING "pending"
#define YAI_DAEMON_POLICY_STALENESS_REFRESH_PENDING "refresh_pending"
#define YAI_DAEMON_POLICY_STALENESS_FRESH_OR_UNKNOWN "fresh_or_unknown"

#define YAI_DAEMON_GRANT_STATE_MISSING_OR_PENDING "missing_or_pending"
#define YAI_DAEMON_GRANT_STATE_PRESENT_NO_EXPIRY "present_no_expiry_v1"

#define YAI_DAEMON_MAX_BINDINGS 32
#define YAI_DAEMON_MAX_OBSERVED 4096
#define YAI_DAEMON_MAX_ACTION_POINTS_PER_BINDING 16

typedef struct yai_daemon_binding_rt {
  char binding_id[96];
  char workspace_id[64];
  char root_path[512];
  char asset_type[64];
  char binding_scope[64];
  char binding_kind[32];
  char observation_scope[128];
  char mediation_scope[128];
  char enforcement_scope[128];
  char mediation_mode[32];
  char action_points_ref[256];
  int action_point_count;
  yai_daemon_action_point_descriptor_t action_points[YAI_DAEMON_MAX_ACTION_POINTS_PER_BINDING];
  char status[32];
  int enabled;
} yai_daemon_binding_rt_t;

typedef struct yai_daemon_observed_asset {
  char key[512];
  char fingerprint[96];
} yai_daemon_observed_asset_t;

typedef struct yai_daemon_local_runtime {
  char source_node_id[96];
  char daemon_instance_id[96];
  char owner_link_id[96];
  char source_enrollment_grant_id[128];
  char owner_trust_artifact_id[128];
  char owner_trust_artifact_token[128];
  char source_policy_snapshot_id[128];
  char source_capability_envelope_id[128];
  char policy_snapshot_version[64];
  char distribution_target_ref[256];
  char delegated_observation_scope[128];
  char delegated_mediation_scope[128];
  char delegated_enforcement_scope[128];
  char owner_socket[512];
  char health_state[32];
  char queue_dir[512];
  char delivered_dir[512];
  char failed_dir[512];
  char observed_index_file[512];
  char bindings_state_file[512];
  char operational_state_file[512];

  yai_daemon_binding_rt_t bindings[YAI_DAEMON_MAX_BINDINGS];
  size_t binding_count;
  yai_daemon_observed_asset_t observed[YAI_DAEMON_MAX_OBSERVED];
  size_t observed_count;

  uint32_t spool_queued;
  uint32_t spool_delivered;
  uint32_t spool_retry_due;
  uint32_t spool_failed;
  uint32_t scan_discovered;
  uint32_t emit_attempts;
  uint32_t emit_success;
  uint32_t emit_failures;
  uint32_t retry_consecutive_failures;
  int64_t runtime_started_epoch;
  int64_t last_observation_epoch;
  int64_t last_successful_emit_epoch;
  int owner_connected;
  int owner_registered;
  int64_t last_owner_contact_epoch;

  char connectivity_state[32];
  char freshness_state[32];
  char spool_pressure_state[32];
  char retry_pressure_state[32];
  char policy_staleness_state[48];
  char grant_validity_state[48];
  char degradation_state[64];
} yai_daemon_local_runtime_t;

int yai_daemon_local_runtime_init(yai_daemon_local_runtime_t *local,
                                  const yai_daemon_config_t *cfg,
                                  const yai_daemon_paths_t *paths,
                                  const char *instance_id,
                                  const char *source_label);

int yai_daemon_local_runtime_start(yai_daemon_local_runtime_t *local);
int yai_daemon_local_runtime_tick(yai_daemon_local_runtime_t *local, uint32_t tick_count);
int yai_daemon_local_runtime_stop(yai_daemon_local_runtime_t *local);

int yai_daemon_local_runtime_health_json(const yai_daemon_local_runtime_t *local,
                                         char *out,
                                         size_t out_cap);
